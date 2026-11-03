import argparse
import json
import re
import zipfile
from collections import Counter
from xml.etree import ElementTree as ET

NS = "{http://schemas.openxmlformats.org/spreadsheetml/2006/main}"
REL = "{http://schemas.openxmlformats.org/officeDocument/2006/relationships}"


def shared_strings(book):
    if "xl/sharedStrings.xml" not in book.namelist():
        return []
    root = ET.fromstring(book.read("xl/sharedStrings.xml"))
    out = []
    for item in root.findall(NS + "si"):
        out.append("".join(node.text or "" for node in item.iter(NS + "t")))
    return out


def sheet_names(book):
    root = ET.fromstring(book.read("xl/workbook.xml"))
    return [(sheet.get("name"), sheet.get(REL + "id")) for sheet in root.iter(NS + "sheet")]


def sheet_paths(book):
    root = ET.fromstring(book.read("xl/_rels/workbook.xml.rels"))
    out = {}
    for item in root:
        target = item.get("Target")
        if not target.startswith("/"):
            target = "xl/" + target.lstrip("./")
        out[item.get("Id")] = target
    return out


def column_index(reference):
    letters = re.match(r"[A-Z]+", reference or "A").group(0)
    value = 0
    for char in letters:
        value = value * 26 + (ord(char) - 64)
    return value - 1


def rows_of(book, path, strings):
    root = ET.fromstring(book.read(path))
    for row in root.iter(NS + "row"):
        cells = {}
        for cell in row.findall(NS + "c"):
            kind = cell.get("t")
            node = cell.find(NS + "v")
            inline = cell.find(NS + "is")
            if kind == "s" and node is not None:
                value = strings[int(node.text)]
            elif kind == "inlineStr" and inline is not None:
                value = "".join(part.text or "" for part in inline.iter(NS + "t"))
            elif node is not None:
                value = node.text
            else:
                value = ""
            cells[column_index(cell.get("r"))] = (value or "").strip()
        if cells:
            yield [cells.get(index, "") for index in range(max(cells) + 1)]


def pick(header, *wanted):
    for index, name in enumerate(header):
        folded = name.lower()
        if any(word in folded for word in wanted):
            return index
    return None


def main():
    parser = argparse.ArgumentParser(description="Count Chilean radio concessions per region from the SUBTEL spreadsheet")
    parser.add_argument("xlsx")
    parser.add_argument("--out", default=None)
    parser.add_argument("--show-header", action="store_true")
    args = parser.parse_args()

    with zipfile.ZipFile(args.xlsx) as book:
        strings = shared_strings(book)
        paths = sheet_paths(book)
        result = {}
        for name, rid in sheet_names(book):
            path = paths.get(rid)
            if not path:
                continue
            rows = list(rows_of(book, path, strings))
            if not rows:
                continue
            header_at = 0
            for index, row in enumerate(rows[:12]):
                if pick(row, "regi") is not None:
                    header_at = index
                    break
            header = rows[header_at]
            if args.show_header:
                print(name, "rows", len(rows), "header", header)
                continue
            region_at = pick(header, "regi")
            service_at = pick(header, "servicio", "tipo")
            holder_at = pick(header, "concesionari", "titular", "nombre")
            if region_at is None:
                continue
            counts = Counter()
            holders = {}
            for row in rows[header_at + 1 :]:
                if len(row) <= region_at or not row[region_at]:
                    continue
                region = row[region_at]
                counts[region] += 1
                if holder_at is not None and len(row) > holder_at:
                    holders.setdefault(region, set()).add(row[holder_at])
            result[name] = {
                "rows": len(rows) - header_at - 1,
                "service_column": header[service_at] if service_at is not None else None,
                "by_region": {
                    region: {"concessions": total, "distinct_holders": len(holders.get(region, ()))}
                    for region, total in sorted(counts.items(), key=lambda pair: -pair[1])
                },
            }
        text = json.dumps(result, ensure_ascii=False, indent=2)
        if args.out:
            open(args.out, "w", encoding="utf-8").write(text)
        for name, found in result.items():
            print(name, "rows", found["rows"], "regions", len(found["by_region"]))


if __name__ == "__main__":
    main()
