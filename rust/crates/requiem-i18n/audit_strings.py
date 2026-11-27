"""Lists string literals that are probably still user-visible and untranslated (run from crates/):
    python i18n/audit_strings.py
Looks for literals handed straight to text/label-style APIs, and `X.to_string()` / format! with words, in UI code."""
import re, glob, os, sys

APIS = r'(?:placeholder|label|text|title|heading|subtitle|message|content|helper_text|styled|TextProps::new|notify|notify_failure|show|description|caption|unavailable|action|header|danger|with_subtitle|with_selected_label|with_group|SelectOption::new|TabItem::new|MenuItem::\w+|ContextItem::\w+|HeadingProps::new|ButtonProps::new|TextLinkProps::new|prompt|tip|tooltip|button|section|field|empty_state|empty_title|row|item|toast|badge|chip|option_row|nav_row|menu_row|action_button|detail_row)'
call = re.compile(r'\b' + APIS + r'\(\s*(?:&)?(?:[A-Za-z_:.()]*,\s*)?"((?:[^"\\]|\\.)*)"')
tolit = re.compile(r'"((?:[^"\\]|\\.)*)"\s*\.(?:to_string|into|to_owned)\(\)')
files = glob.glob('requiem-core/src/**/*.rs', recursive=True) + glob.glob('requiem/src/**/*.rs', recursive=True) + glob.glob('requiem-ui/src/components/*.rs')
skip_file = ('emoji_names_data.rs', 'twemoji_atlas_data.rs', 'playground_stories.rs', 'devtools.rs', 'playground.rs')
out = []
for f in sorted(files):
    f = f.replace(os.sep, '/')
    if f.endswith(skip_file) or '/rest/' in f:
        continue
    in_test = False
    for n, line in enumerate(open(f, encoding='utf8'), 1):
        if '#[cfg(test)]' in line:
            in_test = True
        if in_test or line.strip().startswith('//') or 'i18n::' in line and 'i18n::t!(' in line and '"' not in line.replace('i18n::t!(', ''):
            continue
        for rx in (call, tolit):
            for m in rx.finditer(line):
                t = m.group(1)
                if not re.search(r'[A-Za-z]{2,}', t) or t.startswith(('http', '/', '#')) or re.fullmatch(r'[a-z0-9_\-\.:/#=@]+', t) and ('-' in t or '_' in t or '.' in t):
                    continue
                out.append(f'{f}:{n}: "{t}"')
seen = set()
for o in out:
    if o not in seen:
        seen.add(o)
        print(o)
