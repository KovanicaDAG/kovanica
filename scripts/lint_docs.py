#!/usr/bin/env python3
import re, sys, os
root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
docs = os.path.join(root, "docs")
errors = []
# Product currency naming: KVNC is canonical product ticker (Toni 2026-11-05).
# KVNC is the allowed product ticker (flipped from prior prohibition).
# broken relative links
link_re = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
for dirpath, _, files in os.walk(docs):
    for fn in files:
        if not fn.endswith(".md"):
            continue
        p = os.path.join(dirpath, fn)
        base = os.path.dirname(p)
        with open(p, encoding="utf-8", errors="ignore") as f:
            for i, line in enumerate(f, 1):
                for m in link_re.finditer(line):
                    target = m.group(1)
                    if target.startswith("http") or target.startswith("#") or target.startswith("file:") or target.startswith("mailto:"):
                        continue
                    tpath = os.path.normpath(os.path.join(base, target.split("#")[0]))
                    if not os.path.exists(tpath):
                        errors.append(f"{p}:{i}: broken link -> {target}")
if errors:
    print("\n".join(errors))
    sys.exit(1)
print("docs lint: OK")
