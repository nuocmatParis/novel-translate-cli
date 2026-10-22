"""Independent source inventory. Reads reference books without modifying them."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
from urllib.parse import unquote, urlsplit
import xml.etree.ElementTree as ET
import zipfile


def inventory(filename):
    raw = Path(filename).read_bytes()
    with zipfile.ZipFile(filename) as z:
        names = z.namelist()
        container = ET.fromstring(z.read('META-INF/container.xml'))
        opf = container.find('.//{*}rootfile').attrib['full-path']
        package = ET.fromstring(z.read(opf))
        def resolve(base, href):
            u = urlsplit(href)
            return str(PurePosixPath(base).parent / unquote(u.path)), u.fragment
        manifest = {}
        for n in package.findall('.//{*}manifest/{*}item'):
            p, _ = resolve(opf, n.attrib['href'])
            manifest[n.attrib['id']] = dict(n.attrib, path=p)
        spine = [dict(manifest[n.attrib['idref']], linear=n.get('linear', 'yes'),
                      overrides=dict(n.attrib)) for n in package.findall('.//{*}spine/{*}itemref')]
        docs, navigation = [], []
        for item in manifest.values():
            if item['media-type'] not in ('application/xhtml+xml', 'image/svg+xml', 'application/x-dtbncx+xml'):
                continue
            tree = ET.fromstring(z.read(item['path']))
            images = []
            for n in tree.iter():
                if n.tag.split('}')[-1] in ('img', 'image'):
                    href = n.get('src') or n.get('href') or n.get('{http://www.w3.org/1999/xlink}href')
                    images.append(dict(element=n.tag, href=href))
            docs.append(dict(path=item['path'], image_occurrences=images,
                             ruby=len(tree.findall('.//{*}ruby')),
                             text_nodes=sum(bool(t and t.strip()) for n in tree.iter() for t in (n.text, n.tail)),
                             svg_text=len(tree.findall('.//{http://www.w3.org/2000/svg}text'))))
            if item['media-type'] == 'application/x-dtbncx+xml':
                for point in tree.findall('.//{*}navPoint'):
                    content = point.find('{*}content')
                    navigation.append(dict(label=''.join(point.find('{*}navLabel').itertext()),
                                           src=content.get('src')))
        return dict(source=str(Path(filename).resolve()), sha256=hashlib.sha256(raw).hexdigest(),
                    zip_entries=len(names), xhtml=sum(n.endswith('.xhtml') for n in names),
                    raster_resources=sum(i['media-type'] in ('image/jpeg', 'image/png', 'image/gif') for i in manifest.values()),
                    package=opf, spine=spine, documents=docs, navigation=navigation,
                    extras=sorted(set(names) - {i['path'] for i in manifest.values()} - {opf, 'mimetype', 'META-INF/container.xml'}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('source')
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(inventory(args.source), ensure_ascii=False, indent=2), encoding='utf-8')
