"""Exercise the built CLI and real HTTP adapter with an original scripted EPUB.

Only a loopback HTTP test server is used. These fixed responses verify runtime
plumbing, persistence, and export gates; they do not validate live translation.
"""
import argparse
from datetime import datetime, timezone
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import zipfile


def fixture(destination):
    def xhtml(body):
        return ('<html xmlns="http://www.w3.org/1999/xhtml" '
                'xmlns:epub="http://www.idpf.org/2007/ops"><head><title>試験</title>'
                '</head><body>' + body + '</body></html>')
    files = {
        'mimetype': 'application/epub+zip',
        'META-INF/container.xml': '<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0"><rootfiles><rootfile full-path="OPS/book.opf" media-type="application/oebps-package+xml"/></rootfiles></container>',
        'OPS/book.opf': '<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="uid">urn:uuid:373371e4-b826-4d50-b3ee-0d9fe7d82d11</dc:identifier><dc:title>試験</dc:title><dc:language>ja</dc:language><dc:creator>Original fixture author</dc:creator><meta property="dcterms:modified">2026-10-12T00:00:00Z</meta></metadata><manifest><item id="main" href="main.xhtml" media-type="application/xhtml+xml" properties="svg"/><item id="notes" href="notes.xhtml" media-type="application/xhtml+xml"/><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="image" href="image.svg" media-type="image/svg+xml"/></manifest><spine><itemref idref="main"/></spine></package>',
        'OPS/main.xhtml': xhtml('<h1 id="c1">第一章</h1><p id="p1">猫は来た。</p><p><ruby>名<rt>な</rt></ruby>と<strong>強調</strong>。<a epub:type="noteref" href="notes.xhtml#n1">注</a></p><svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 10 10"><image width="10" height="10" xlink:href="image.svg"/></svg><h1 id="c2">第二章</h1><p>雨が降るが、彼は走った。</p>'),
        'OPS/notes.xhtml': xhtml('<aside epub:type="footnote" id="n1"><p>注釈。<a href="main.xhtml#p1">戻る</a></p></aside>'),
        'OPS/nav.xhtml': xhtml('<nav epub:type="toc"><ol><li><a href="main.xhtml#c1">第一章</a></li><li><a href="main.xhtml#c2">第二章</a></li></ol></nav>'),
        'OPS/image.svg': '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10" fill="blue"/></svg>',
    }
    with zipfile.ZipFile(destination, 'w') as archive:
        for name, content in files.items():
            archive.writestr(name, content.encode('utf-8'),
                             compress_type=zipfile.ZIP_STORED if name == 'mimetype' else zipfile.ZIP_DEFLATED)


TRANSLATIONS = {
    '試験': 'Thử nghiệm', '第一章': 'Chương một', '第二章': 'Chương hai',
    '猫は来た。': 'Con mèo đã đến.',
    '名と⟦t0⟧強調⟦t1⟧。⟦t2⟧注⟦t3⟧': 'Tên và ⟦t0⟧nhấn mạnh⟦t1⟧. ⟦t2⟧Chú thích⟦t3⟧',
    '雨が降るが、彼は走った。': 'Dù trời mưa, cậu ấy đã chạy.',
    '注釈。⟦t0⟧戻る⟦t1⟧': 'Chú thích. ⟦t0⟧Quay lại⟦t1⟧',
}


def translated(source):
    if source in TRANSLATIONS:
        return TRANSLATIONS[source]
    # Navigation labels may be extracted inside paired inline link tokens.
    for japanese in ('試験', '第一章', '第二章'):
        if source == '⟦t0⟧' + japanese + '⟦t1⟧':
            return '⟦t0⟧' + TRANSLATIONS[japanese] + '⟦t1⟧'
    raise ValueError('Unexpected synthetic source: ' + repr(source))


def main():
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    parser = argparse.ArgumentParser()
    parser.add_argument('--cli', required=True)
    parser.add_argument('--epubcheck', required=True)
    parser.add_argument('--java-bin', required=True)
    parser.add_argument('--fonts')
    parser.add_argument('--output', default='scratch/acceptance/offline-http')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    out = (root / args.output).resolve()
    if not out.is_relative_to(root / 'scratch'):
        parser.error('Acceptance artifacts must be inside scratch/')
    out.mkdir(parents=True, exist_ok=True)
    cli = Path(args.cli).resolve()
    env = os.environ.copy()
    env['PATH'] = str(Path(args.java_bin).resolve()) + os.pathsep + env.get('PATH', '')
    calls, errors, commands = [], [], []

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_args):
            pass

        def do_POST(self):
            try:
                if self.path != '/v1/chat/completions':
                    raise ValueError('Unexpected endpoint ' + self.path)
                request = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
                payload = None
                for message in request['messages']:
                    content = message.get('content', '')
                    if isinstance(content, list):
                        content = ''.join(item.get('text', '') for item in content)
                    try:
                        decoded = json.loads(content)
                    except (ValueError, TypeError):
                        continue
                    if isinstance(decoded, dict) and 'owned_units' in decoded:
                        payload = decoded
                if payload is None:
                    raise ValueError('Missing owned source payload')
                blocks = payload['owned_units']
                mode = payload['mode']
                if mode == 'review':
                    response = {'reviewed': [{'id': unit['id'], 'target_hash': hashlib.sha256(unit['target'].encode('utf-8')).hexdigest()} for unit in payload['targets']], 'issues': [], 'ruby_decisions': [dict(index=i, base=ruby['base'], reading=ruby['reading'], decision='phonetic', target_quote='', reason='scripted synthetic fixture only') for block in blocks for i, ruby in enumerate(block.get('ruby', []))]}
                else:
                    response = {'units': [{'id': block['id'], 'target': translated(block['source'])} for block in blocks], 'uncertainties': [], 'continuity_candidates': [], 'rule_candidates': []}
                calls.append(dict(mode=mode, ids=[block['id'] for block in blocks], model=request['model']))
                body = json.dumps({'id': 'local-script', 'object': 'chat.completion', 'model': 'scripted', 'choices': [{'index': 0, 'finish_reason': 'stop', 'message': {'role': 'assistant', 'content': json.dumps(response, ensure_ascii=False)}}], 'usage': {'prompt_tokens': 10, 'completion_tokens': 5, 'total_tokens': 15}}).encode('utf-8')
                self.send_response(200)
            except Exception as error:
                errors.append(str(error))
                body = json.dumps({'error': {'message': str(error)}}).encode('utf-8')
                self.send_response(400)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Content-Length', str(len(body)))
            self.end_headers()
            self.wfile.write(body)

    server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    workspace = Path(tempfile.mkdtemp(prefix='Bộ thử HTTP ', dir=out))

    def save(name, value):
        (out / name).write_text(json.dumps(value, ensure_ascii=False, indent=2), encoding='utf-8')

    def command(name, *argv, allowed=0):
        proc = subprocess.run([str(cli), *map(str, argv)], cwd=root / 'scratch', env=env, capture_output=True, timeout=180)
        (out / (name + '.stdout.json')).write_bytes(proc.stdout)
        (out / (name + '.stderr.txt')).write_bytes(proc.stderr)
        commands.append(dict(check=name, argv=[str(cli), *map(str, argv)], exit_code=proc.returncode))
        save('commands.json', commands)
        print(name, proc.returncode, flush=True)
        if proc.returncode != allowed:
            raise RuntimeError(name + ': ' + proc.stderr.decode('utf-8', errors='replace')[:2000])
        return json.loads(proc.stdout) if proc.stdout.strip() else None

    try:
        command('init', 'init', workspace, '--series', 'original-fixture', '--title', 'Bộ thử HTTP')
        source = workspace / 'nguồn Nhật.epub'
        fixture(source)
        provider = {'provider': 'local', 'model': 'scripted', 'providers': {'local': {'type': 'openai', 'allow_no_api_key': True, 'base_url': f'http://127.0.0.1:{server.server_port}/v1', 'models': [{'name': 'scripted', 'context_window': 32768, 'json_schema': False}]}}}
        (workspace / 'provider.json').write_text(json.dumps(provider), encoding='utf-8')
        config = workspace / 'workspace.yaml'
        config.write_text(config.read_text(encoding='utf-8') + '\nprovider_config: provider.json\n', encoding='utf-8')
        command('register', 'volume', 'add', '--workspace', workspace, '--id', 'v1', '--order', '1', '--source', source)
        book = command('import', 'import', '--workspace', workspace, '--volume', 'v1')
        base = ['--workspace', workspace, '--volume', 'v1']
        images = command('images', 'image', 'list', *base)
        for i, image in enumerate(images):
            command(f'image-{i}', 'image', 'assess', *base, '--id', image['id'], '--disposition', 'no-text', '--reason', 'Original synthetic blue rectangle has no text')
        command('translate', 'translate', *base)
        count = len(calls)
        command('resume', 'resume', *base)
        if len(calls) != count or errors:
            raise AssertionError('Resume repeated valid requests or script failed: ' + repr(errors))
        command('verify', 'verify', *base)
        command('missing-checker-gate', 'export', *base, allowed=2)
        if (workspace / 'Output/v1_VI.epub').exists():
            raise AssertionError('Final was published without EPUBCheck')
        text = config.read_text(encoding='utf-8')
        text = text.replace('    template: sample-vi', '    template: sample-vi\n    epubcheck: ' + json.dumps(str(Path(args.epubcheck).resolve()).replace('\\', '/')))
        if args.fonts:
            text = text.replace('    template: sample-vi', '    template: sample-vi\n    font_directory: ' + json.dumps(str(Path(args.fonts).resolve()).replace('\\', '/')))
        config.write_text(text, encoding='utf-8')
        final = command('final-export', 'export', *base)
        report = final['report']
        if not report['structure_checked'] or report['accepted'] != report['eligible'] or 'pass' not in report['format']:
            raise AssertionError('Final gates did not pass: ' + repr(report))
        (out / 'scripted-final.epub').write_bytes((workspace / 'Output/v1_VI.epub').read_bytes())
        save('scripted-final.report.json', report)
        status = command('status', 'status', '--workspace', workspace, '--json')
        if status['usage']['requests'] != count:
            raise AssertionError('Usage does not match actual HTTP requests')
        skill = workspace / 'Skills/SKILL.md'
        skill.write_text(skill.read_text(encoding='utf-8') + '\nUpdated fixture convention.\n', encoding='utf-8')
        stale = command('stale-status', 'status', '--workspace', workspace, '--json')
        if stale['volumes'][0]['accepted'] != 0 or not stale['volumes'][0]['stale']:
            raise AssertionError('Changed skill did not invalidate accepted status')
        save('acceptance.json', dict(time=datetime.now(timezone.utc).isoformat(), cli_sha256=hashlib.sha256(cli.read_bytes()).hexdigest(), workspace=str(workspace), requests=count, format=report['format'], structure=report['structure_checked'], accepted=report['accepted'], eligible=report['eligible'], resume_no_requests=True, missing_checker_blocks_final=True, skill_change_stale=True, real_model_acceptance='pending — scripted loopback responses only', source_hash=book['source_hash']))
        return 0
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
        save('http-calls.json', calls)
        save('http-errors.json', errors)


if __name__ == '__main__':
    sys.exit(main())
