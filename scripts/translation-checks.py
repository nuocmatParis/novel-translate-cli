"""Run Go checks with exact argv and retain machine-readable verification evidence."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys


def main():
    # Windows consoles may default to cp1252 despite UTF-8 artifacts.
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    parser = argparse.ArgumentParser()
    parser.add_argument('--go', default='go')
    parser.add_argument('--race', action='store_true')
    parser.add_argument('--output', default='scratch/acceptance/checks')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    out = (root / args.output).resolve()
    out.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    for key, directory in [('GOCACHE', 'scratch/gocache'), ('GOPATH', 'scratch/gopath'),
                           ('GOTMPDIR', 'scratch/go-build'), ('TEMP', 'scratch/test-temp'), ('TMP', 'scratch/test-temp')]:
        p = root / directory
        p.mkdir(parents=True, exist_ok=True)
        env[key] = str(p)
    env['GOWORK'] = 'off'
    commands = [('vet', [args.go, 'vet', './...']),
                ('tests', [args.go, 'test', '-ldflags=-s -w', '-buildvcs=false', '-count=1', './...']),
                ('build', [args.go, 'build', '-trimpath', '-ldflags=-s -w -X main.version=dev', '-o', str(root / 'scratch/novel-translate-cli.exe'), './cmd/novel-translate-cli']),
                ('diff', ['git', 'diff', '--check'])]
    if args.race:
        commands.insert(2, ('race', [args.go, 'test', '-race', '-buildvcs=false', '-count=1', './internal/translation/...']))
    results = []
    for name, argv in commands:
        proc = subprocess.run(argv, cwd=root, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        (out / (name + '.txt')).write_bytes(proc.stdout)
        print(name, 'PASS' if proc.returncode == 0 else 'FAIL', flush=True)
        if proc.returncode:
            print(proc.stdout.decode('utf-8', errors='replace'), flush=True)
        results.append(dict(check=name, command=argv, exit_code=proc.returncode))
    (out / 'results.json').write_text(json.dumps(results, indent=2), encoding='utf-8')
    return int(any(r['exit_code'] for r in results))


if __name__ == '__main__':
    sys.exit(main())
