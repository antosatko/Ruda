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

def main():
    start_time = time.time()

    factorial(252)
    fib(30)

    end_time = time.time()
    elapsed_time = (end_time - start_time) * 1000  # Convert to milliseconds

    print(f"Elapsed time: {elapsed_time:.2f} milliseconds")

if __name__ == "__main__":
    main()
