#!/usr/bin/env python3
"""Assemble the SeqLock Wortschatz from the extraction workflow's JSON result.
Produces a styled HTML (for PDF) and a markdown source. Each example carries its
English translation (the new requirement vs. the async-semaphore edition).

Usage: python3 assemble_wortschatz.py <workflow_result.json> <out.html> <out.md>
"""
import json, sys, html, re

result_path, out_html, out_md = sys.argv[1], sys.argv[2], sys.argv[3]
obj = json.load(open(result_path))
parts = obj['result'] if isinstance(obj, dict) and 'result' in obj else obj

# --- dedupe by headword, keeping earliest T-tag (part order) ---
seen = {}
order = []
for part in parts:
    tag = part['tag']
    for e in part.get('entries', []):
        key = e['headword'].strip().lower()
        if key in seen:
            continue
        e = dict(e); e['tag'] = tag
        seen[key] = e
        order.append(e)

CATS = [
    ('noun',     'Substantive', 'nouns'),
    ('verb',     'Verben', 'verbs'),
    ('adj',      'Adjektive & Adverbien', 'adjectives and adverbs'),
    ('idiom',    'Redewendungen & feste Wendungen', 'idioms and set phrases'),
    ('marker',   'Textstruktur & Signalwörter', 'discourse markers'),
    ('loanword', 'Englische Lehnwörter', 'English loanwords and the gender German gives them'),
]

def esc_code(s):
    # escape, then turn `code` backtick spans into <code>
    return re.sub(r'`([^`]+)`', lambda m: '<code>' + m.group(1) + '</code>', html.escape(s))

def mark_bold(s):
    # convert **kw** to <b>, `code` to <code>, escape the rest
    parts_ = re.split(r'(\*\*.*?\*\*)', s)
    out = ''
    for seg in parts_:
        if seg.startswith('**') and seg.endswith('**'):
            out += '<b class="kw">' + esc_code(seg[2:-2]) + '</b>'
        else:
            out += esc_code(seg)
    return out

def mark_en(s):
    # EN example line: bold the **keyword**, render `code`, escape rest (stays muted).
    parts_ = re.split(r'(\*\*.*?\*\*)', s)
    out = ''
    for seg in parts_:
        if seg.startswith('**') and seg.endswith('**'):
            out += '<b class="enkw">' + esc_code(seg[2:-2]) + '</b>'
        else:
            out += esc_code(seg)
    return out

# Curated core vocabulary — "learn these first". Matched against headwords.
CORE = [
    'Payload', 'Sequenzzähler', 'Zähler', 'Schreiber', 'Leser', 'Fence', 'Fenster',
    'Bump', 'Kante', 'zerreißen', 'torn read', 'Data Race', 'Pod', 'Marker-Trait',
    'atomar', 'gerade', 'ungerade', 'Wort', 'Maschinenwort', 'Randbedingung',
    'Miri', 'loom', 'Cache line', 'Core', 'stabil', 'erzwingen',
]

def split_headword(hw):
    m = re.match(r'^(der|die|das)\s+(.*)$', hw.strip())
    if m:
        return f'<span class="art">{m.group(1)}</span> {html.escape(m.group(2))}'
    return html.escape(hw.strip())

def entry_html(e):
    meta = html.escape(e.get('meta','') or '')
    metabox = f'<div class="meta">{meta}</div>' if meta else ''
    return f'''<div class="e">
  <div class="l">
    <div class="hw">{split_headword(e["headword"])}</div>
    <div class="tags">{metabox}<span class="tt">{e["tag"]}</span></div>
  </div>
  <div class="r">
    <div class="mean">{html.escape(e.get("en_meaning",""))}</div>
    <div class="de">{mark_bold(e.get("de_example",""))}</div>
    <div class="en">{mark_en(e.get("en_example",""))}</div>
  </div>
</div>'''

def section(de_title, en_title, count, rows_html):
    return f'''<section>
  <div class="sec"><h2>{de_title}</h2><span class="subt">{en_title}</span><span class="cnt">{count}</span></div>
  {rows_html}
</section>'''

# --- Kernwortschatz: curated core, matched against headwords, in CORE order ---
core_entries, used = [], set()
for needle in CORE:
    for e in order:
        hw = e['headword']
        if needle.lower() in hw.lower() and hw.lower() not in used:
            core_entries.append(e); used.add(hw.lower()); break
kern_html = section('Kernwortschatz', 'the words the whole series runs on — learn these first',
                    len(core_entries),
                    '<p class="note">Diese Wörter kommen in jedem Teil vor. Wer sie kann, versteht die Struktur jedes Absatzes.</p>\n' +
                    '\n'.join(entry_html(e) for e in core_entries))

# --- Grammatikmuster (hand-authored, SeqLock examples) ---
GRAMMAR = [
 ('Nominalisierter Infinitiv — das Lesen', 'WORTBILDUNG',
  'Jeden Infinitiv großschreiben, und er wird ein neutrales Substantiv = englisches «-ing».',
  [('Das <b>Lesen</b> ist der Hot Path.', 'The reading is the hot path.'),
   ('Das <b>Zerreißen</b> lassen wir geschehen und fangen es ab.', 'We let the tearing happen and catch it.'),
   ('Das <b>Schreiben</b> klammert der Writer mit zwei Bumps.', 'The writer brackets the writing with two bumps.')]),
 ('Erweitertes Partizipialattribut', 'SATZBAU',
  'Englisch hängt einen Relativsatz an; Deutsch schiebt das Ganze zwischen Artikel und Substantiv. Von hinten lesen: erst das Substantiv, dann nach links auspacken.',
  [('ein halb <b>geschriebener</b> Wert', 'a half-written value'),
   ('die vom Writer gerade <b>geänderten</b> Bytes', 'the bytes the writer is currently changing'),
   ('der zwischen zwei Bumps <b>geschriebene</b> payload', 'the payload written between two bumps')]),
 ('Konditional ohne «wenn» — Verberststellung', 'SATZBAU',
  '«wenn» weglassen und das Verb an Position eins setzen — kompakter und formeller, überall in den Regeln.',
  [('<b>Ist</b> s1 ungerade, liest der Reader gar nicht erst.', 'If s1 is odd, the reader doesn\'t even read.'),
   ('<b>Greift</b> die Kopie auch nur ein Byte von Write N auf, ist s2 ≠ s1.', 'If the copy grabs even one byte of write N, s2 ≠ s1.')]),
 ('Dativ-Verben', 'FÄLLE',
  'Einige Verben nehmen das Dativobjekt, wo Englisch ein schlichtes Objekt hat.',
  [('Wir <b>trauen</b> dem grünen Test nicht.', 'We don\'t trust the green test.'),
   ('Der Code <b>gehorcht</b> dem Protokoll aus geraden und ungeraden Bumps.', 'The code obeys the protocol of even and odd bumps.'),
   ('Der Beweis <b>folgt</b> daraus.', 'The proof follows from it.')]),
 ('Trennbare Verben im Hauptsatz', 'SATZBAU',
  'Die Vorsilbe springt ans Ende des Satzes; im Nebensatz bleibt sie dran.',
  [('Der Reader <b>liest</b> den Zähler zweimal <b>ab</b>.', 'The reader reads the counter off twice.'),
   ('…, weil der Writer den Schreibvorgang <b>einklammert</b>.', '…because the writer brackets the write.')]),
 ('Komposita — von rechts nach links lesen', 'WORTBILDUNG',
  'Das letzte Element ist der Kopf: es trägt Genus und Kernbedeutung. Alles links davon grenzt es ein.',
  [('der Sequenz|zähler', 'sequence + counter → sequence counter'),
   ('das Maschinen|wort', 'machine + word'),
   ('die Rand|bedingung', 'edge + condition → constraint'),
   ('die Cache-|Kohärenz', 'cache + coherence')]),
 ('«man» als allgemeines Subjekt', 'STIL',
  'Wo Englisch «you» oder ein Passiv schreibt, schreibt Deutsch «man» — nie ein Mann, sondern «one, anybody».',
  [('Setzt <b>man</b> alle vier Kanten nebeneinander, wählt sich das Werkzeug fast von selbst.', 'Lay the four edges side by side and the tool almost picks itself.'),
   ('<b>Man</b> setzt am ungeraden Bump an, nicht am geraden.', 'One keys on the odd bump, not the even one.')]),
]
def grammar_block():
    out = []
    for name, tag, expl, exs in GRAMMAR:
        rows = ''.join(f'<div class="gx"><div class="gde">{de}</div><div class="gen">{en}</div></div>' for de, en in exs)
        out.append(f'<div class="gp"><div class="gh"><span class="gname">{name}</span><span class="gt">{tag}</span></div><p class="note">{expl}</p>{rows}</div>')
    return section('Grammatikmuster', 'the patterns that make the text feel hard', len(GRAMMAR), '\n'.join(out))

sections_html = [kern_html]
counts = {}
for cat, de_title, en_title in CATS:
    items = [e for e in order if e['category'] == cat]
    counts[cat] = len(items)
    if not items:
        continue
    rows = '\n'.join(entry_html(e) for e in items)
    sections_html.append(f'''<section>
  <div class="sec"><h2>{de_title}</h2><span class="subt">{en_title}</span><span class="cnt">{len(items)}</span></div>
  {rows}
</section>''')

total = len(order)
loan = counts.get('loanword', 0)

HEAD = f'''<!doctype html><html lang="de"><head><meta charset="utf-8">
<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Source+Serif+4:ital,opsz,wght@0,8..60,400;0,8..60,600;1,8..60,400&family=JetBrains+Mono:wght@400;600&display=swap">
<style>
  :root{{--ink:#1b1c1e;--muted:#6b7280;--line:#e3e6ea;--accent:#0b6570;--art:#8a94a0;--kw:#0b6570;--box:#eef1f4}}
  *{{box-sizing:border-box}}
  body{{margin:0;color:var(--ink);font-family:"Source Serif 4",Georgia,serif;font-size:10.5pt;line-height:1.5;background:#fff}}
  .wrap{{max-width:900px;margin:0 auto;padding:8mm}}
  .eyebrow{{font-family:"JetBrains Mono",monospace;font-size:8pt;letter-spacing:.14em;text-transform:uppercase;color:var(--muted)}}
  h1{{font-weight:600;font-size:26pt;margin:.15em 0 .1em;letter-spacing:-.01em}}
  h1 em{{font-style:italic;color:var(--accent)}}
  .lede{{font-size:11pt;color:#374151;max-width:60em;margin:.2em 0 1em}}
  .lede em{{font-style:italic}}
  .counts{{font-size:9pt;color:var(--muted);border-top:1px solid var(--line);border-bottom:1px solid var(--line);padding:7px 0;margin:0 0 14px}}
  .counts b{{color:var(--ink)}}
  .legend{{background:var(--box);border-radius:6px;padding:10px 14px;font-size:8.6pt;color:#374151;margin:0 0 20px}}
  .legend b{{color:var(--ink)}}
  .sec{{display:flex;align-items:baseline;gap:12px;border-bottom:2px solid var(--ink);padding-bottom:4px;margin:22px 0 12px;break-after:avoid}}
  .sec h2{{font-weight:600;font-size:15pt;margin:0}}
  .sec .subt{{font-size:9pt;color:var(--muted);flex:1}}
  .sec .cnt{{font-family:"JetBrains Mono",monospace;font-size:9pt;color:var(--muted)}}
  .e{{display:grid;grid-template-columns:170px 1fr;gap:18px;padding:9px 0;border-bottom:1px solid var(--line);break-inside:avoid}}
  .hw{{font-weight:600;font-size:11pt}}
  .hw .art{{font-style:italic;font-weight:400;color:var(--art)}}
  .tags{{display:flex;align-items:center;gap:6px;margin-top:4px}}
  .meta{{font-family:"JetBrains Mono",monospace;font-size:7.6pt;color:var(--muted)}}
  .tt{{font-family:"JetBrains Mono",monospace;font-size:7.2pt;color:var(--muted);border:1px solid var(--line);border-radius:3px;padding:0 4px}}
  .mean{{font-size:9.6pt;color:#111827;margin-bottom:3px}}
  .de{{font-style:italic;font-size:9.6pt;border-left:2px solid var(--line);padding-left:9px;color:#1f2937}}
  .de .kw{{font-style:normal;font-weight:600;color:var(--kw)}}
  .en{{font-size:8.8pt;color:var(--muted);padding-left:9px;margin-top:2px}}
  .en .enkw{{font-weight:600;color:#4b5563}}
  .note{{color:var(--muted);font-size:8.8pt;margin:2px 0 10px}}
  .gp{{break-inside:avoid;padding:10px 0 6px;border-bottom:1px solid var(--line)}}
  .gh{{display:flex;align-items:baseline;gap:10px}}
  .gname{{font-weight:600;font-size:11.5pt}}
  .gt{{font-family:"JetBrains Mono",monospace;font-size:7.2pt;letter-spacing:.12em;color:var(--accent);border:1px solid var(--line);border-radius:3px;padding:1px 6px}}
  .gx{{display:grid;grid-template-columns:1fr 1fr;gap:16px;padding:3px 0}}
  .gde{{font-style:italic;font-size:9.6pt;color:#1f2937}}
  .gde b{{font-style:normal;font-weight:600;color:var(--kw)}}
  .gen{{font-size:9.2pt;color:var(--muted)}}
  footer{{margin-top:24px;border-top:2px solid var(--ink);padding-top:8px;font-size:8pt;color:var(--muted)}}
</style></head><body><div class="wrap">
<div class="eyebrow">EIN SEQLOCK ENTWERFEN · TEIL 1–4 · DEUTSCH → ENGLISCH</div>
<h1>SeqLock-<em>Wortschatz</em></h1>
<p class="lede">Jedes Wort, jede Wendung und jedes Grammatikmuster aus der deutschen Serie, das über A2 hinausgeht — jeweils mit englischer Bedeutung <em>und</em> einem Beispielsatz aus dem Text, das Stichwort hervorgehoben. <b>Neu in dieser Ausgabe:</b> jeder Beispielsatz trägt seine englische Übersetzung direkt darunter.</p>
<div class="counts"><b>{total}</b> Vokabeleinträge · <b>{loan}</b> Lehnwörter · Beispielsätze wörtlich aus <b>plan/blog/seqlock/de/</b>, englische Zeile aus <b>../en/</b> · T1–T4 = Teil des ersten Vorkommens</div>
<div class="legend"><b>Legende.</b> <i>der</i> maskulin · <i>die</i> feminin · <i>das</i> neutrum · ¨-e Plural mit Umlaut · kein Pl. kein Plural · Stammformen bei starken Verben: Präteritum · Partizip II · (+ Dat.) Dativ-Verb · Die englische Zeile unter jedem Beispiel ist der Paralleltext aus der englischen Fassung.</div>
'''

FOOT = '''<footer>Generiert aus der deutschen Fassung der Serie „Ein SeqLock entwerfen" · Beispielsätze wörtlich aus dem Text, englische Zeile aus der Parallelfassung.</footer>
</div></body></html>'''

open(out_html, 'w').write(HEAD + '\n'.join(sections_html) + grammar_block() + FOOT)

# --- markdown source ---
md = [f'# SeqLock-Wortschatz — Deutsch → Englisch',
      '',
      f'Jedes A2+ Wort aus der deutschen Serie, mit englischer Bedeutung und einem Beispielsatz aus dem Text — **diesmal mit englischer Übersetzung des Beispiels darunter**. {total} Einträge; T1–T4 = Teil des ersten Vorkommens.',
      '']
for cat, de_title, en_title in CATS:
    items = [e for e in order if e['category'] == cat]
    if not items: continue
    md.append(f'## {de_title} — {en_title} ({len(items)})\n')
    for e in items:
        meta = f' · {e["meta"]}' if e.get('meta') else ''
        md.append(f'**{e["headword"]}**{meta} · `{e["tag"]}` — {e.get("en_meaning","")}')
        md.append(f'> DE: {e.get("de_example","")}')
        md.append(f'> EN: {e.get("en_example","")}')
        md.append('')
open(out_md, 'w').write('\n'.join(md))
print(f'assembled {total} entries -> {out_html} + {out_md}')
for c,_,_ in CATS: print(f'  {c}: {counts.get(c,0)}')
