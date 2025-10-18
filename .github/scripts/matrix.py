#!/usr/bin/env python3

import itertools
import json
import subprocess

output = subprocess.check_output(
    "yq -ojson '.jobs.check-and-test.strategy.matrix' .github/workflows/ci.yml",
    shell=True,
)

matrix = json.loads(output)

exclude = matrix.pop("exclude", [])
include = matrix.pop("include", [])

keys = sorted(matrix.keys())
values = [matrix[key] for key in keys]

excluded = {tuple((key, item[key]) for key in keys) for item in exclude}
included = {tuple((key, item[key]) for key in keys) for item in include}

generated = {tuple(zip(keys, items)) for items in itertools.product(*values)}

final = (generated - excluded) | included
final_list = [{key: value for key, value in items} for items in sorted(final)]

items = []
for item in final_list:
    os = item["os"]
    rust = item["rust"]
    rustc_bootstrap = item["rustc_bootstrap"]

    bootstrap = "" if rust == "nightly" or rustc_bootstrap == "0" else "-bootstrap"

    items.append(f"code-coverage-report-{rust}-{os}{bootstrap}")


print(",".join(items))
