"""Lists candidate hardcoded UI strings (run from crates/): python i18n/extract_strings.py OUT.txt"""
import re, glob, sys, os, collections

out = []
files = (
    glob.glob('requiem-core/src/**/*.rs', recursive=True)
    + glob.glob('requiem/src/**/*.rs', recursive=True)
    + glob.glob('requiem-ui/src/components/*.rs')
)
skip_ctx = re.compile(r'log::|println!|eprintln!|panic!|expect\(|assert|#\[|Error::|anyhow|bail!|\.header\(|env::var|include_|Path::|SELECT|tracing|warn!|info!|debug!|error!|trace!')
lit = re.compile(r'"((?:[^"\\]|\\.)*)"')
for f in sorted(files):
    f = f.replace(os.sep, '/')
    if f.endswith(('emoji_names_data.rs', 'twemoji_atlas_data.rs', 'playground_stories.rs')):
        continue
    for n, line in enumerate(open(f, encoding='utf8'), 1):
        if line.strip().startswith('//') or skip_ctx.search(line):
            continue
        for m in lit.finditer(line):
            t = m.group(1)
            if not re.search(r'[A-Za-z]{2,}', t) or t.startswith(('http', '/', '#', '.')):
                continue
            if re.fullmatch(r'[a-z0-9_\-\.:/#=@]+', t):
                continue
            if not (t[0].isupper() or ' ' in t):
                continue
            out.append(f'{f}:{n}: {t}')
open(sys.argv[1], 'w', encoding='utf8').write('\n'.join(out))
print(len(out))
for k, v in collections.Counter(o.split(':')[0] for o in out).most_common():
    print(v, k)
