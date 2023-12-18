import time

def factorial(n):
    if n == 0:
        return 1
    return n * factorial(n - 1)

def fib(n):
    if n == 0:
        return 0
    if n == 1:
        return 1
    return fib(n - 1) + fib(n - 2)

def fibonacci(n):
    if n <= 0:
        return "Please enter a positive integer for Fibonacci sequence."
    elif n == 1:
        return [0]
    elif n == 2:
        return [0, 1]
    else:
        fib_sequence = [0, 1]
        for i in range(2, n):
            fib_sequence.append(fib_sequence[-1] + fib_sequence[-2])
        return fib_sequence

def main():
    start_time = time.time()

    i = 0
    while i < 100000:
        fibonacci(93)
        i += 1

    end_time = time.time()
    elapsed_time = (end_time - start_time) * 1000  # Convert to milliseconds

    print(f"Elapsed time: {elapsed_time:.2f} milliseconds")

if __name__ == "__main__":
    main()
