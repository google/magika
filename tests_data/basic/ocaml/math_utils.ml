(* Mathematical utility module in OCaml *)

let rec factorial (n : int) : int =
  if n < 0 then invalid_arg "factorial: negative argument"
  else if n = 0 then 1
  else n * factorial (n - 1)

let rec gcd (a : int) (b : int) : int =
  if b = 0 then abs a
  else gcd b (a mod b)

let is_prime (n : int) : bool =
  if n <= 1 then false
  else
    let rec check d =
      if d * d > n then true
      else if n mod d = 0 then false
      else check (d + 1)
    in
    check 2

let safe_div (x : float) (y : float) : float option =
  if y = 0.0 then None
  else Some (x /. y)
