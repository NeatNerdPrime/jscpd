# Type-3 clones (near-miss)

A near-miss clone is a copy with a few lines inserted, removed or changed in
the middle. A token-window detector sees it as two shorter exact clones with
a gap between them. `--max-gap-lines N` (config key `maxGapLines`, off at
`0`) merges clones of the same file pair whose fragments follow each other in
both files with at most `N` unmatched lines between them, and reports one
clone of kind `similar` with a `similarity` value (matched tokens over the
tokens of the longer merged span).

Commands run from the repository root at default thresholds; the lines after
`#` are what the console reporter prints.

| Directory | Gap | Default scan | `--max-gap-lines 1` | `--max-gap-lines 2` |
|-----------|-----|--------------|---------------------|---------------------|
| `inserted-line/` | 1 line | 2 exact clones | 1 similar clone | 1 similar clone |
| `wide-gap/` | 2 lines | 2 exact clones | 2 exact clones | 1 similar clone |

## `inserted-line/` — one inserted guard

`save-account.js` is `save-user.js` with one `if (...) throw` line added in
the middle.

```bash
jscpd fixtures/type3-demo/inserted-line
# Clone found (javascript)
#  - save-account.js [1:1 - 6:5] (6 lines, 60 tokens)
#    save-user.js [1:1 - 6:6]
# Clone found (javascript)
#  - save-account.js [6:66 - 12:2] (7 lines, 100 tokens)
#    save-user.js [5:55 - 11:2]
# Found 2 clones.

jscpd fixtures/type3-demo/inserted-line --max-gap-lines 1
# Clone found (javascript, similar ~0.91)
#  - save-account.js [1:1 - 12:2] (12 lines, 157 tokens)
#    save-user.js [1:1 - 11:2]
# Found 1 clones.
```

The two halves overlap on their boundary token (the `;` that ends line 5 of
`save-user.js`), so the merged clone matches 157 tokens, not 160.

## `wide-gap/` — a three-line block, two lines of gap

`place-order-guarded.js` is `place-order.js` with a three-line `if` block
inserted. The first exact clone ends on the `if` line and the second starts
on the closing brace, leaving two unmatched lines between them, so a limit
of 1 keeps the halves apart and a limit of 2 merges them.

```bash
jscpd fixtures/type3-demo/wide-gap --max-gap-lines 1
# Found 2 clones.

jscpd fixtures/type3-demo/wide-gap --max-gap-lines 2
# Clone found (javascript, similar ~0.91)
#  - place-order-guarded.js [1:1 - 15:2] (15 lines, 172 tokens)
#    place-order.js [1:1 - 12:2]
# Found 1 clones.
```

## Whole directory

```bash
jscpd fixtures/type3-demo
# Found 4 clones.

jscpd fixtures/type3-demo --max-gap-lines 2
# Found 2 clones.

jscpd fixtures/type3-demo --max-gap-lines 2 --reporters json,silent --output report
# each entry in "duplicates" has "kind": "similar" and "similarity": 0.91…
```

Merging only joins clones the exact run already reported, so it never adds a
match that was not there. Merged spans differ from the exact fragments, so a
run with `--max-gap-lines` needs its own `--baseline` file.
