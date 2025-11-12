**Definitions**

```
  x,y  : booleans
  a,b  : bytes
a == b : boolean, true iff a equals b
a != b : boolean, false iff a equals b
 nz(x) : byte, 0 iff NOT x, unspecified otherwise (nonzero)
 ao(x) : byte, 0xff iff x, unspecified otherwise (all ones)
 nm(x) : byte, 0xff iff x, 0 iff NOT x (normal)
 A = B : A equals B
A <- B : A can be replaced by B
   |   : boolean or bitwise OR
   &   : boolean or bitwise AND
   !   : boolean or bitwise NOT
   ^   : boolean or bitwise XOR
```

**Rules**

```
# nz is only distributive over OR
nz(x | y) = nz(x) | nz(y)
# ao is only distributive over AND
ao(x & y) = ao(x) & ao(y)
# invert between ao and nz
ao(!x) = !nz(x)

# nm is distributive over both OR and AND
nm(x | y) = nm(x) | nm(y)
nm(x & y) = nm(x) & nm(y)
nm(!x) = !nm(x)

# replace nz or ao with nm
nz(x) <- nm(x)
ao(x) <- nm(x)

# instructions
and(a, b) = a & b
andn(a, b) = !a & b
or(a, b) = a | b
xor(a, b) = a ^ b = nz(a != b)
cmpeq(a, b) = nm(a == b)
min(nz(x), nz(y)) = nz(x & y)
max(ao(x), ao(y)) = ao(x | y)

# normalize by comparing with zero
nm(!x) = cmpeq(nz(x), 0)

# normalize by comparing with all ones
nm(x) = cmpeq(ao(x), 0xff)
```

**Boolean expressions**

```
# if % is allowed
valid = allowed & !invalid_pct
invalid_pct = pct & !(hexdig_1 & hexdig_2)

# if % is not allowed
valid = allowed_alt | valid_pct
valid_pct = pct & hexdig_1 & hexdig_2

pct = byte == b'%'
```

**Given expressions**

```
# calculated by two shuffles and bitwise AND/OR
# where table and mask can be pre-inverted:
# nz(x) = table & mask
# ao(x) = table | !mask
# nz(!x) = !table & mask
# ao(!x) = !table | !mask
nz(x), ao(x), nz(!x), ao(!x) where x can be any one of
allowed, allowed_alt, hexdig_1, and hexdig_2

# constants
b'%', 0, 0xff
```

**Task**

Replace `nz(valid)` or `nz(!valid)` with an expression
consisting only of the given expressions and instructions,
with the minimum number of instructions.

Before:

```
nz(!valid) <- nm(!valid)
  = nm(!allowed | invalid_pct)
  = or(nm(!allowed), nm(invalid_pct))

nm(!allowed) = cmpeq(nz(allowed), 0)

nm(invalid_pct)
  = nm(pct & !(hexdig_1 & hexdig_2))
  = and(nm(pct), nm(!hexdig_1 | !hexdig_2))

nm(pct) = nm(byte == b'%')
  = cmpeq(byte, b'%')

nm(!hexdig_1 | !hexdig_2)
  = or(nm(!hexdig_1), nm(!hexdig_2))

nm(!hexdig_1) = cmpeq(nz(hexdig_1), 0)
nm(!hexdig_2) = cmpeq(nz(hexdig_2), 0)
```

With a total number of 7 instructions
(2 or, 1 and, 4 cmpeq).

After:

```
nz(!valid)
  = nz(!allowed | invalid_pct)
  = or(nz(!allowed), nz(invalid_pct))

nz(invalid_pct)
  = nz(pct & !(hexdig_1 & hexdig_2))
 <- min(nm(pct), nz(!hexdig_1 | !hexdig_2))
  = min(cmpeq(byte, b'%'), or(nz(!hexdig_1), nz(!hexdig_2)))
```

With a total number of 4 instructions
(2 or, 1 min, 1 cmpeq).

Shift left version:

```
# it seems that % must be allowed
valid = allowed & !disallowed_after_pct
disallowed_after_pct = !hexdig & (after_pct_1 | after_pct_2)
```

```
nz(!valid)
  = nz(!allowed | disallowed_after_pct)
  = or(nz(!allowed), nz(disallowed_after_pct))

nz(disallowed_after_pct)
  = nz(!hexdig & (after_pct_1 | after_pct_2))
  = min(nz(!hexdig), or(nz(after_pct_1), nz(after_pct_2)))
```