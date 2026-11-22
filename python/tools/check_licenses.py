# SPDX-License-Identifier: GPL-3.0-or-later
"""Every package in requirements.lock must have a GPL-3 compatible license (ADR-0001).

    <env>/bin/python -I python/tools/check_licenses.py python/requirements.lock

Reads the installed metadata (License-Expression, else the license classifiers, else
License) of each locked package in the environment running this script.
"""

import re
import sys
from importlib import metadata

ALLOW = {
    "MIT", "MIT-0", "ISC", "BSD-2-Clause", "BSD-3-Clause", "0BSD", "Apache-2.0", "MPL-2.0", "Zlib",
    "PSF-2.0", "Python-2.0", "Unlicense", "CC0-1.0", "BSL-1.0", "LGPL-2.1-or-later", "LGPL-3.0-or-later",
    "GPL-3.0-or-later", "GPL-2.0-or-later", "Apache-2.0 WITH LLVM-exception", "HPND", "MIT-CMU", "NCSA",
}
# Trove classifiers that mean an allowed license.
CLASSIFIERS = {
    "MIT License": "MIT", "BSD License": "BSD-3-Clause", "Apache Software License": "Apache-2.0",
    "ISC License (ISCL)": "ISC", "Python Software Foundation License": "PSF-2.0",
    "Mozilla Public License 2.0 (MPL 2.0)": "MPL-2.0", "The Unlicense (Unlicense)": "Unlicense",
    "GNU Lesser General Public License v3 or later (LGPLv3+)": "LGPL-3.0-or-later",
}


def allowed(expr):
    e = expr.replace("(", " ").replace(")", " ")
    return any(all(t.strip() in ALLOW for t in re.split(r"\s+AND\s+", alt)) for alt in re.split(r"\s+OR\s+", e))


def license_of(dist):
    m = dist.metadata
    if m.get("License-Expression"):
        return m["License-Expression"], allowed(m["License-Expression"])
    found = [c.split(" :: ")[-1] for c in m.get_all("Classifier") or [] if c.startswith("License :: OSI Approved")]
    if found:
        return ", ".join(found), all(f in CLASSIFIERS for f in found)
    text = (m.get("License") or "").strip().splitlines()[0] if m.get("License") else ""
    return text or "unknown", allowed(text) if text else False


def main(lock):
    names = re.findall(r"^([A-Za-z0-9_.\-]+)==", open(lock).read(), re.M)
    bad = []
    for name in names:
        try:
            lic, ok = license_of(metadata.distribution(name))
        except metadata.PackageNotFoundError:
            bad.append(f"{name}: not installed in this environment")
            continue
        if not ok:
            bad.append(f"{name}: {lic}")
    if bad:
        print(f"packages without an allowed license ({len(bad)} of {len(names)}):\n  " + "\n  ".join(bad), file=sys.stderr)
        return 1
    print(f"Python licenses ok ({len(names)} packages)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1] if len(sys.argv) > 1 else "python/requirements.lock"))
