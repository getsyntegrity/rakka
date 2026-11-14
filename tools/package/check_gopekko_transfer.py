#!/usr/bin/env python3
"""Verifica el índice de transferencia de goPekko y las copias presentes.

Falla si: hay ids duplicados, una fila no tiene destino ni exclusión justificada,
una copia presente difiere del blob de origen, o un archivo copiado no está en el
índice. Las copias de bloques todavía no integrados se informan como pendientes.
"""
import csv, glob, hashlib, os, re, sys

ROOT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..')
BASE = os.path.join(ROOT, 'docs/reference/gopekko')
rows = []
for f in sorted(glob.glob(os.path.join(BASE, 'index/gopekko-transfer-index-*.tsv'))):
    with open(f, encoding='utf-8', newline='') as fh:
        rows += list(csv.DictReader(fh, delimiter='\t', quoting=csv.QUOTE_NONE))
fail, pending, ok, links_bad = [], 0, 0, 0
ids = [r['id'] for r in rows]
if len(ids) != len(set(ids)): fail.append('ids duplicados')
paths = {r['source_path'] for r in rows}
if len(paths) != len(rows): fail.append('source_path duplicado')

def blob(data): return hashlib.sha1(b'blob %d\0' % len(data) + data).hexdigest()

present = set()
for r in rows:
    t = r['treatment']
    if t not in ('copiar', 'vincular', 'excluir'): fail.append(r['id'] + ': tratamiento invalido')
    if t == 'excluir' and not r['why']: fail.append(r['id'] + ': exclusion sin motivo')
    if t == 'vincular' and not r['destination'].startswith('https://github.com/getsyntegrity/gopekko/blob/'): fail.append(r['id'] + ': enlace permanente invalido')
    if t != 'copiar': continue
    dest = r['destination']
    n = int(r['parts'])
    if n == 1: files = [dest]
    else: files = ['%s.part-%02d-of-%02d' % (dest.replace(' (partes)', ''), k + 1, n) for k in range(n)]
    got = [os.path.join(ROOT, p) for p in files if os.path.isfile(os.path.join(ROOT, p))]
    present.update(files)
    if not got: pending += 1; continue
    if len(got) != n: fail.append(r['id'] + ': copia incompleta %d/%d partes' % (len(got), n)); continue
    data = b''.join(open(p, 'rb').read() for p in got)
    if blob(data) != r['blob_sha']: fail.append(r['id'] + ': contenido distinto del blob de origen ' + r['source_path'])
    else:
        ok += 1
        if r['source_path'].endswith('.md'):
            text = data.decode('utf-8')
            for m in re.finditer(r'\]\((?!https?:|#|mailto:)([^)\s#]+)', text):
                tgt = os.path.normpath(os.path.join(os.path.dirname(r['source_path']), m.group(1)))
                if tgt not in paths and not tgt.startswith('..'): links_bad += 1

for p in glob.glob(os.path.join(BASE, '**/*'), recursive=True):
    rel = os.path.relpath(p, ROOT)
    if os.path.isfile(p) and not rel.startswith(('docs/reference/gopekko/index/',)) and os.path.basename(p) not in ('README.md', 'NOTICE.gopekko') and rel not in present:
        # README.md de origen tambien existen bajo la misma ruta espejo
        if rel[len('docs/reference/gopekko/'):] not in paths and rel not in present:
            fail.append('archivo copiado fuera del indice: ' + rel)
print('filas=%d copias_verificadas=%d copias_pendientes=%d enlaces_a_destino_fuera_del_indice=%d fallos=%d' % (len(rows), ok, pending, links_bad, len(fail)))
for f in fail[:50]: print('FALLO', f)
sys.exit(1 if fail else 0)
