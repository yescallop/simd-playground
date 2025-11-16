# Predicate Mask Algebra for SIMD

## Notations

```
# variables
 x,y,z : booleans
  a,b  : unsigned fixed-width integers with maximum value MAX
  A,B  : sets of integers

# mask types
 nz(x) : set of integers, {1, .., MAX} iff x, {0} otherwise (nonzero)
 ao(x) : set of integers, {MAX} iff x, {0, .., MAX-1} otherwise (all ones)
 nm(x) : set of integers, {MAX} iff x, {0} otherwise (normal)

# relations
 A = B : A equals B
A <- B : A can be substituted with B (B is a subset of A)

# operators (lifted pointwise to sets)
a == b : boolean, true iff a equals b
a != b : boolean, true iff a does not equal b
   |   : boolean or bitwise OR
   &   : boolean or bitwise AND
   !   : boolean or bitwise NOT
   ^   : boolean or bitwise XOR
```

## Rules

```
# nz is only distributive over OR
nz(x | y) = nz(x) | nz(y)
# ao is only distributive over AND
ao(x & y) = ao(x) & ao(y)
# invert between ao and nz
ao(!x) = !nz(x), nz(!x) = !ao(x)

# nm is distributive over both OR and AND
nm(x | y) = nm(x) | nm(y)
nm(x & y) = nm(x) & nm(y)
# invert nm directly
nm(!x) = !nm(x)

# factorize nz(AND) / ao(OR) with nm
nz(x & y) = nm(x) & nz(y) = nz(x) & nm(y)
ao(x | y) = nm(x) | ao(y) = ao(x) | nm(y)

# substitute nz or ao with nm
nz(x) <- nm(x)
ao(x) <- nm(x)

# instructions (lifted pointwise to sets)
or(a, b) = a | b
and(a, b) = a & b
xor(a, b) = a ^ b
 => xor(a, MAX) = !a
    nz(a != b) <- xor(a, b)
andn(a, b) = !a & b
 => andn(a, MAX) = !a

# normalize with cmpeq
cmpeq(a, b) = MAX iff a == b, 0 otherwise
 => nm(a == b) = cmpeq(a, b)
    nm(!x) = cmpeq(nz(x), 0)
    nm(x) = cmpeq(ao(x), MAX)

# factorize nz(AND) / ao(OR) with min/max
min(a, b) = a iff a < b, b otherwise
 => nz(x & y) = min(nz(x), nz(y))
max(a, b) = a iff a > b, b otherwise
 => ao(x | y) = max(ao(x), ao(y))

# select with blend
blend(a, b, c) = b iff c has highest bit set, a otherwise
 => nz((!z & x) | (z & y)) = blend(nz(x), nz(y), nm(z))
    ao((!z & x) | (z & y)) = blend(ao(x), ao(y), nm(z))
    nm((!z & x) | (z & y)) = blend(nm(x), nm(y), nm(z))
```

## Example task

Given the following expressions:

```
# if % is allowed
valid = allowed & !invalid_pct
invalid_pct = pct & !(hexdig_1 & hexdig_2)

pct = byte == b'%'
```

Given the following terms, variables, and constants:

```
# terms calculated by two shuffles and bitwise AND/OR
# where table and mask can be pre-inverted:
# nz(x) = table & mask
# ao(x) = table | !mask
# nz(!x) = !table & mask
# ao(!x) = !table | !mask
nz(x), ao(x), nz(!x), ao(!x) where x is among
allowed, hexdig_1, hexdig_2

# variables
byte

# constants
b'%', 0, 0xff
```

Substitute `nz(valid)` or `nz(!valid)` with an expression
consisting only of the given terms and instructions,
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
  = and(nm(pct), nz(!hexdig_1 | !hexdig_2))
  = and(cmpeq(byte, b'%'), or(nz(!hexdig_1), nz(!hexdig_2)))
```

With a total number of 4 instructions
(2 or, 1 and, 1 cmpeq).

Or use the alternative form:

```
# regardless of whether % is allowed or not
valid = (!pct & allowed) | (pct & hexdig_1 & hexdig_2)

nz(!valid)
  = nz((!pct & !allowed) | (pct & !(hexdig_1 & hexdig_2)))
  = blend(nz(!allowed), nz(!(hexdig_1 & hexdig_2)), nm(pct))
  = blend(nz(!allowed), or(nz(!hexdig_1), nz(!hexdig_2)), cmpeq(byte, b'%'))
```

With a total number of 3 instructions
(1 or, 1 cmpeq, 1 blend)

Shift left version:

```
# it seems that % must be allowed
valid = allowed & !disallowed_after_pct
disallowed_after_pct = !hexdig & after_pct
```

```
nz(!valid)
  = nz(!allowed | disallowed_after_pct)
  = or(nz(!allowed), nz(!hexdig & after_pct))
  = or(nz(!allowed), and(nz(!hexdig), nm(after_pct)))

nz(valid)
  = nz(allowed & !disallowed_after_pct)
  = min(nz(allowed), nz(hexdig | !after_pct))
```

Or use the alternative form:

```
valid = (!after_pct & allowed) | (after_pct & hexdig)

nz(valid)
  = nz((!after_pct & allowed) | (after_pct & hexdig))
  = blend(nz(allowed), nz(hexdig), nm(after_pct))
```
