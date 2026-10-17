#!/usr/bin/env python3
"""Check local Markdown/HTML links and GitHub heading anchors in README and Deck guide."""
from pathlib import Path
import re
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]


def anchors(text):
    result, counts = set(re.findall(r'(?:id|name)=[\"\']([^\"\']+)', text)), {}
    for heading in re.findall(r'^#{1,6}\s+(.+?)\s*#*$', text, re.M):
        heading = re.sub(r'\[([^]]+)\]\([^)]*\)', r'\1', heading)
        slug = re.sub(r'[^\w\- ]', '', heading.lower()).replace(' ', '-')
        count = counts.get(slug, 0)
        counts[slug] = count + 1
        result.add(slug + (f'-{count}' if count else ''))
    return result


def main():
    total = 0
    for relative in ['README.md', 'docs/steam-deck-guide.md']:
        source = ROOT / relative
        text = source.read_text()
        links = re.findall(r'!?\[[^\]]*\]\(([^\s)]+)(?:\s+[^)]*)?\)', text)
        links += re.findall(r'(?:src|href)=[\"\']([^\"\']+)', text)
        for link in links:
            url = urlsplit(link)
            if url.scheme or url.netloc:
                continue
            target = (source.parent / unquote(url.path)).resolve() if url.path else source
            assert target.exists(), f'{relative}: missing {link}'
            if url.fragment:
                assert unquote(url.fragment) in anchors(target.read_text()), f'{relative}: missing anchor {link}'
            total += 1
    print(f'PASS: {total} local links/anchors in README and Steam Deck guide')


if __name__ == '__main__':
    main()
