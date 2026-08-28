#!/usr/bin/env python3
# Rust code-card generator -> One Dark HTML (rendered to PNG by render.mjs).
import re, html, sys, os

KW = set("pub fn let mut self const unsafe for in if else while loop return match as impl struct enum trait where use mod type ref move dyn break continue".split())
TYPES = set("T Self usize u8 u16 u32 u64 i32 i64 f32 f64 bool AtomicUsize Ordering MaybeUninit UnsafeCell SeqLock Pod Copy Relaxed Acquire Release AcqRel B256 Vec Option Some None Result Ok Err".split())

TOKEN = re.compile(r'''(?P<num>\b\d[\d_]*\b)|(?P<ident>[A-Za-z_][A-Za-z0-9_]*!?)|(?P<op>->|=>|::|[{}()\[\];,.:&*=%<>+\-/|])|(?P<ws>\s+)|(?P<other>.)''')

def hl_code(s):
    out=[]
    toks=list(TOKEN.finditer(s))
    for i,m in enumerate(toks):
        t=m.group()
        if m.lastgroup=='num': out.append(f'<span class="n">{html.escape(t)}</span>')
        elif m.lastgroup=='ident':
            nxt=''
            for j in range(i+1,len(toks)):
                if toks[j].lastgroup=='ws': continue
                nxt=toks[j].group(); break
            if t.endswith('!'): out.append(f'<span class="f">{html.escape(t)}</span>')
            elif t in KW: out.append(f'<span class="k">{html.escape(t)}</span>')
            elif t in TYPES: out.append(f'<span class="t">{html.escape(t)}</span>')
            elif nxt=='(': out.append(f'<span class="f">{html.escape(t)}</span>')
            else: out.append(f'<span class="p">{html.escape(t)}</span>')
        else:
            out.append(html.escape(t))
    return ''.join(out)

def hl_line(line):
    # split off a // comment (snippets have no // inside strings)
    ci=line.find('//')
    if ci>=0:
        code, comment = line[:ci], line[ci:]
    else:
        code, comment = line, ''
    # strings in code
    parts=re.split(r'("(?:[^"\\]|\\.)*")', code)
    codehtml=''
    for k,pt in enumerate(parts):
        if k%2==1: codehtml+=f'<span class="s">{html.escape(pt)}</span>'
        else: codehtml+=hl_code(pt)
    if comment:
        cls='c'
        if '①' in comment or '②' in comment: cls='w'
        elif '③' in comment or '④' in comment: cls='rd'
        elif '✗' in comment or 'BUG' in comment or 'tears' in comment or 'UB' in comment or 'WRONG' in comment: cls='bad'
        codehtml+=f'<span class="{cls}">{html.escape(comment)}</span>'
    return codehtml

THEME='''<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:ital,wght@0,400;0,500;0,700;1,400&display=swap">
<style>
html,body{margin:0;background:#0d1117}
.fig{display:inline-block;padding:44px;background:radial-gradient(120% 120% at 20% 0%,#1b2330 0%,#0d1117 70%)}
.card{width:max-content;min-width:560px;max-width:1040px;background:#282c34;border-radius:12px;box-shadow:0 22px 60px rgba(0,0,0,.55),0 0 0 1px rgba(255,255,255,.04);overflow:hidden;font-family:'JetBrains Mono',monospace}
.bar{display:flex;align-items:center;gap:8px;padding:14px 18px;background:#21252b}
.dot{width:12px;height:12px;border-radius:50%}.r{background:#ec6a5e}.y{background:#f4bf4f}.g{background:#61c554}
.file{margin-left:12px;color:#7f8896;font-size:13px;letter-spacing:.02em}
pre{margin:0;padding:22px 26px 26px;font-size:15px;line-height:1.72;color:#abb2bf;white-space:pre}
.ln{display:inline-block;width:2.4ch;color:#4b5263;user-select:none}
.k{color:#c678dd}.t{color:#e5c07b}.f{color:#61afef}.n{color:#d19a66}.s{color:#98c379}
.c{color:#7f848e;font-style:italic}.w{color:#e5a04e;font-style:italic}.rd{color:#56b6c2;font-style:italic}
.bad{color:#ec6a5e;font-style:italic}.p{color:#abb2bf}
</style>'''

def card(title, code, numbers=True):
    lines=code.split('\n')
    body=[]
    for i,ln in enumerate(lines,1):
        pre = f'<span class="ln">{i}</span>' if numbers else ''
        body.append(pre+hl_line(ln))
    inner='\n'.join(body)
    return f'''<!doctype html><html><head><meta charset="utf-8">{THEME}</head><body>
<div class="fig"><div class="card">
<div class="bar"><span class="dot r"></span><span class="dot y"></span><span class="dot g"></span><span class="file">{html.escape(title)}</span></div>
<pre>{inner}</pre></div></div></body></html>'''

if __name__=='__main__':
    outdir='plan/blog/seqlock/img/src/cards'
    os.makedirs(outdir, exist_ok=True)
    import cards_data
    for name,(title,code,num) in cards_data.CARDS.items():
        open(f'{outdir}/{name}.html','w').write(card(title,code,num))
        print('card',name)
