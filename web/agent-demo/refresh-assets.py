import base64
import hashlib
import html
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlparse


class SnippetIntegrity(HTMLParser):
    def __init__(self, root):
        super().__init__(convert_charrefs=False)
        self.root = root
        self.output = []

    def handle_starttag(self, tag, attributes):
        values = dict(attributes)
        path = unquote(urlparse(values.get("href", "")).path)
        if tag == "link" and "snippets/" in path and "integrity" in values:
            asset = self.root / "snippets" / path.split("snippets/", 1)[1]
            asset = asset.resolve()
            if not asset.is_relative_to(self.root.resolve()):
                raise ValueError("Invalid generated snippet path")
            digest = base64.b64encode(hashlib.sha384(asset.read_bytes()).digest()).decode("ascii")
            attributes = [(name, "sha384-" + digest if name == "integrity" else value) for name, value in attributes]
            rendered = " ".join(name if value is None else f'{name}="{html.escape(value, quote=True)}"' for name, value in attributes)
            self.output.append(f"<{tag} {rendered}>")
        else:
            self.output.append(self.get_starttag_text())

    def handle_startendtag(self, tag, attributes):
        self.output.append(self.get_starttag_text())

    def handle_endtag(self, tag):
        self.output.append(f"</{tag}>")

    def handle_data(self, data):
        self.output.append(data)

    def handle_entityref(self, name):
        self.output.append(f"&{name};")

    def handle_charref(self, name):
        self.output.append(f"&#{name};")

    def handle_comment(self, data):
        self.output.append(f"<!--{data}-->")

    def handle_decl(self, data):
        self.output.append(f"<!{data}>")


root = Path(__file__).parent / "dist"
parser = SnippetIntegrity(root)
index = root / "index.html"
parser.feed(index.read_text())
parser.close()
index.write_text("".join(parser.output))