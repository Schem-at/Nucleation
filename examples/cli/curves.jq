# Sample a formula with t in [0,1]. Radius is a brush radius in blocks.
# Output uses the Java structure SNBT layout accepted by nucleation convert.
def curve($steps; $radius; $block; point):
  if ($steps < 1 or $steps != ($steps|floor) or $radius < 0) then
    error("steps must be a positive integer and radius must be non-negative")
  else . end
  | ($radius|floor) as $r
  | [range(-$r; $r+1) as $x | range(-$r; $r+1) as $y | range(-$r; $r+1) as $z
     | select($x*$x + $y*$y + $z*$z <= $radius*$radius) | [$x,$y,$z]] as $brush
  | [range(0; $steps+1) | (. / $steps | point)
     | if type == "array" and length == 3 and all(.[]; type == "number" and isfinite)
       then map(round) else error("point must return three finite coordinates") end]
  | unique
  | [.[] as $p | $brush[] as $b | [range(3) as $i | $p[$i]+$b[$i]]] | unique
  | . as $points
  | [range(3) as $i | map(.[$i]) | min] as $lo
  | [range(3) as $i | (map(.[$i])|max) - $lo[$i]+1] as $size
  | {DataVersion:4671, size:$size, palette:[$block], entities:[],
     data:[$points[] as $p | {pos:[range(3) as $i | $p[$i]-$lo[$i]], state:$block}]};

# One revolution in radians, plus math notation for the formula argument.
def t: . * 2 * 3.141592653589793;
def sin($angle): $angle | sin;
def cos($angle): $angle | cos;
