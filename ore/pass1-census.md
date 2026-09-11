# OpenGGS furnace census

Convention: `ore/convention.pass1.ggs`

| quantity | value |
|---|---|
| ore facts enumerated | 34326 |
| melted rows | 21854 |
| residual rows | 12472 |
| conserved | true |
| functions | 181 |
| translation units | 55 |

## Melted rows by concern

| concern | rows |
|---|---|
| State | 21854 |

## Slag shapes, ranked — the repeat signal

| shape_id | reason | count |
|---|---|---|
| `19e8851fae228c8e` | EventKindNotInConvention[ScopeExit] | 3487 |
| `19e8861fae228e41` | EventKindNotInConvention[ScopeEnter] | 3487 |
| `19e87c1fae227d43` | EventKindNotInConvention[Condition] | 1688 |
| `19e87b1fae227b90` | EventKindNotInConvention[Branch] | 1617 |
| `19e87d1fae227ef6` | EventKindNotInConvention[Call] | 1260 |
| `19e87e1fae2280a9` | EventKindNotInConvention[Decl] | 353 |
| `19e88a1fae22950d` | EventKindNotInConvention[Cast] | 346 |
| `19e8891fae22935a` | EventKindNotInConvention[Param] | 139 |
| `19e8881fae2291a7` | EventKindNotInConvention[Break] | 69 |
| `19e8871fae228ff4` | EventKindNotInConvention[Return] | 26 |

## Function shape

NOT REPORTED: this convention melted 0 `Control` rows, so control density is 0.0 for every function by construction and would classify the whole corpus as data-shaped. Classify a control kind first.
