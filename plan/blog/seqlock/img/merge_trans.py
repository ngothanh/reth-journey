import json, sys
# merge full-translation output into entries, write in assemble's {result:[...]} shape
entries_path, trans_path, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
parts = json.load(open(entries_path))
tobj = json.load(open(trans_path))
trans = tobj['result'] if isinstance(tobj, dict) and 'result' in tobj else tobj
tmap = {t['part']: {it['i']: it['en'] for it in t.get('items', [])} for t in trans}
missing = 0
for part in parts:
    m = tmap.get(part['part'], {})
    for i, e in enumerate(part.get('entries', [])):
        if i in m and m[i].strip():
            e['en_example'] = m[i]
        else:
            missing += 1
json.dump({'result': parts}, open(out_path, 'w'), ensure_ascii=False)
print(f"merged; {missing} entries kept old en_example (no translation returned)")
