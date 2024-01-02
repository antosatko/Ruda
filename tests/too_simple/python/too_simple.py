import sys 
import time
def main():
    start = time.time()
    n = 10_000_000_000
    sum = 0.0
    flip = -1.0
    i = 0
    while i <= n:
        flip *= -1.0        
        sum += flip / (2*i - 1)                                      
        i += 1
    end = time.time()
    print("elapsed time = %.9f" % (end - start))
    print("%.9f" % (sum*4.0))
main() 