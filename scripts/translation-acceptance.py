"""Opt-in read-only real EPUB/research acceptance, without model credentials.

Never declares Japanese source echo to be a Vietnamese translation. All outputs
are inside the requested artifact directory; supplied books/skills stay read-only.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import zipfile
import xml.etree.ElementTree as ET

def main():
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    parser = argparse.ArgumentParser()
    parser.add_argument('--cli', required=True)
    parser.add_argument('--source', required=True)
    parser.add_argument('--template', required=True)
    parser.add_argument('--skill')
    parser.add_argument('--epubcheck', required=True)
    parser.add_argument('--java-bin', required=True)
    parser.add_argument('--research-url')
    parser.add_argument('--volume-id', default='vol01')
    parser.add_argument('--volume-order', type=int, default=1)
    parser.add_argument('--output', default='scratch/acceptance/real-epub')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    out = (root / args.output).resolve()
    if not out.is_relative_to(root):
        parser.error('Artifacts must stay inside this checkout')
    out.mkdir(parents=True, exist_ok=True)
    cli = Path(args.cli).resolve()
    workspace = out / 'workspace'
    env = os.environ.copy()
    env['PATH'] = str(Path(args.java_bin).resolve()) + os.pathsep + env.get('PATH', '')
    source = Path(args.source).resolve()
    template = Path(args.template).resolve()
    hashes = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in (source, template)}
    results = []

    def save(name, data):
        (out / name).write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding='utf-8')

    def command(name, *argv, allowed=(0,)):
        # Execute from a different cwd; exact argv preserves Unicode/spaces.
        proc = subprocess.run([str(cli), *map(str, argv)], cwd=root / 'scratch', env=env,
                              capture_output=True, timeout=180)
        (out / (name + '.stdout.json')).write_bytes(proc.stdout)
        (out / (name + '.stderr.txt')).write_bytes(proc.stderr)
        results.append(dict(check=name, argv=[str(cli), *map(str, argv)], exit_code=proc.returncode))
        print(name, proc.returncode, flush=True)
        save('commands.json', results)
        if proc.returncode not in allowed:
            raise RuntimeError(name + ': ' + proc.stderr.decode('utf-8', errors='replace')[:1500])
        return json.loads(proc.stdout) if proc.stdout.strip() else None

    # Import module from the source file's actual hyphenated filename.
    save('source-inventory.json', inventory(source))
    save('template-inventory.json', inventory(template))
    save('environment.json', dict(time=datetime.now(timezone.utc).isoformat(),
                                cli=str(cli), cli_sha256=hashlib.sha256(cli.read_bytes()).hexdigest(),
                                epubcheck=str(Path(args.epubcheck).resolve()),
                                java_bin=str(Path(args.java_bin).resolve()),
                                model_acceptance='pending — no provider/model configuration',
                                source_hashes=hashes))
    if not (workspace / 'workspace.yaml').exists():
        command('init', 'init', workspace, '--series', 'rezero', '--title', 'Re:Zero — kiểm thử cấu trúc')
        command('register', 'volume', 'add', '--workspace', workspace, '--id', args.volume_id, '--order', args.volume_order, '--source', source)
    book = command('import', 'import', '--workspace', workspace, '--volume', args.volume_id)
    save('extraction-report.json', dict(source_hash=book['source_hash'], text_nodes=book['text_nodes'],
                                      covered_text_nodes=book['covered_text_nodes'],
                                      text_attributes=book['text_attributes'], covered_text_attributes=book['covered_text_attributes'],
                                      chapters=book['chapters'], diagnostics=book['diagnostics']))
    if args.skill:
        command('skill-import', 'skill', 'import', '--workspace', workspace, '--source', Path(args.skill).resolve())
    # Fonts come only from the user-selected template and are not committed.
    fonts = workspace / '.translation_work' / 'template-fonts'
    fonts.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(template) as z:
        for name in z.namelist():
            base = Path(name).name
            if base in ['LiberationSerif-Regular.ttf', 'LiberationSerif-Bold.ttf',
                        'LiberationSerif-Italic.ttf', 'LiberationSerif-BoldItalic.ttf']:
                (fonts / base).write_bytes(z.read(name))
    config_path = workspace / 'workspace.yaml'
    text = config_path.read_text(encoding='utf-8')
    if '    epubcheck:' not in text:
        text += '    epubcheck: ' + json.dumps(str(Path(args.epubcheck).resolve()).replace('\\', '/')) + '\n'
    if len(list(fonts.glob('*.ttf'))) == 4 and '    font_directory:' not in text:
        text += '    font_directory: .translation_work/template-fonts\n'
    if args.research_url and 'research_sources:' not in text:
        text += '\nresearch_sources:\n  - id: official-character\n    api: fetch\n    base_url: ' + json.dumps(args.research_url) + '\n    edition: anime\n    kind: official\n'
    config_path.write_text(text, encoding='utf-8')
    command('status', 'status', '--workspace', workspace, '--json')
    preview = command('preview', 'export', '--workspace', workspace, '--volume', args.volume_id, '--preview')
    save('export-report.json', preview['report'])
    save('output-inventory.json', inventory(preview['output']))
    # Final must remain blocked because no Japanese source units were translated.
    command('final-gate', 'export', '--workspace', workspace, '--volume', args.volume_id, allowed=(2,))
    mains = [ch['id'] for ch in book['chapters'] if ch['kind'] == 'chapter']
    if len(mains) >= 2:
        command('range-preview', 'export', '--workspace', workspace, '--volume', args.volume_id, '--preview',
                '--from', mains[0], '--to', mains[1])
    if args.research_url:
        command('research-live', 'skill', 'research', '--workspace', workspace, '--volume', args.volume_id, '--term', 'スバル', allowed=(0, 1))
    unchanged = all(hashlib.sha256(Path(p).read_bytes()).hexdigest() == h for p, h in hashes.items())
    save('acceptance.json', dict(original_inputs_unchanged=unchanged, extraction_covered=book['text_nodes'] == book['covered_text_nodes'] and book['text_attributes'] == book['covered_text_attributes'],
                                 structural_preview=preview['report']['structure_checked'],
                                 format=preview['report']['format'], translated_units=0,
                                 semantic_acceptance='pending — source echo is not translation',
                                 scope='source import/full preview/first two main chapters preview',
                                 commands=results))
    if not unchanged:
        raise RuntimeError('Original input changed')


if __name__ == '__main__':
    # Hyphenated script filename is deliberately independent from Go reader.
    import importlib.util
    spec = importlib.util.spec_from_file_location('translation_inventory', Path(__file__).with_name('translation-inventory.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    inventory = module.inventory
    main()
