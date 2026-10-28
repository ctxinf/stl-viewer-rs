#!/usr/bin/env python3
"""Generate small, deterministic format compatibility fixtures."""
from pathlib import Path
from zipfile import ZipFile, ZipInfo, ZIP_DEFLATED

out = Path(__file__).resolve().parents[1] / 'tests/fixtures'
ns = 'http://schemas.microsoft.com/3dmanufacturing/core/2015/02'
prod = 'http://schemas.microsoft.com/3dmanufacturing/production/2015/06'
mesh = '<mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="1" y="0" z="0"/><vertex x="0" y="1" z="0"/></vertices><triangles><triangle v1="0" v2="1" v3="2"/></triangles></mesh>'
def model(resources, build='', unit='millimeter'):
    return f'<model xmlns="{ns}" xmlns:p="{prod}" unit="{unit}"><resources>{resources}</resources><build>{build}</build></model>'
def archive(name, root, parts=None, missing_thumbnail=False):
    entries = {'[Content_Types].xml': '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/></Types>', '_rels/.rels': '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/3dmodel.model" Id="r1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>', '3D/3dmodel.model': root, **(parts or {})}
    if missing_thumbnail:
        entries['_rels/.rels'] = entries['_rels/.rels'].replace('</Relationships>', '<Relationship Target="Metadata/plate_1.png" Id="thumb" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail"/></Relationships>')
    with ZipFile(out / name, 'w') as z:
        for path, data in entries.items():
            info = ZipInfo(path, (2026, 1, 1, 0, 0, 0))
            info.compress_type = ZIP_DEFLATED
            z.writestr(info, data)
archive('instances.3mf', model(f'<object id="1">{mesh}</object><object id="2"><components><component objectid="1" transform="1 0 0 0 1 0 0 0 1 0 2 0"/></components></object>', '<item objectid="2" transform="1 0 0 0 1 0 0 0 1 3 0 0"/><item objectid="1"/>', 'inch'))
archive('external.3mf', model('<object id="2"><components><component objectid="1" p:path="/3D/Objects/part.model"/></components></object>', '<item objectid="2"/>'), {'3D/Objects/part.model': model(f'<object id="1">{mesh}</object>')})
archive('cycle.3mf', model('<object id="1"><components><component objectid="1"/></components></object>', '<item objectid="1"/>'))
archive('bad-index.3mf', model(f'<object id="1">{mesh.replace("v3=\"2\"", "v3=\"9\"")}</object>', '<item objectid="1"/>'))
(out / 'quad.obj').write_text('mtllib missing.mtl\nv 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf -4 -3 -2 -1\n')

archive('missing-thumbnail.3mf', model(f'<object id="1">{mesh}</object>', '<item objectid="1"/>'), missing_thumbnail=True)
