fun isPrime(n: Int): Boolean {
    if (n < 2) return false
    for (i in 2..Math.sqrt(n.toDouble()).toInt()) {
        if (n % i == 0) return false
    }
    return true
}

fun main() {
    val primes = (2..50).filter { isPrime(it) }
    println(primes.joinToString(" "))
}
