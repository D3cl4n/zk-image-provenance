import math


# find the distribution of greyscale values across the space [0-256)
def count():
    y_distribution = [0] * 256
    
    # iterate over all the rgb pixel channel combinations
    for r in range(256):
        for g in range(256):
            for b in range(256):
                y = math.floor((30*r + 58*g + 11*b)/100)
                y_distribution[y] += 1

    return y_distribution


# find the number of colliding pairs without repeating order (A, B) = (B, A) here
def count_colliding_pairs(y_distribution):
    total = 0
    for y in range(256):
        count_y = y_distribution[y]
        total += (count_y * (count_y - 1)) 

    return total


# main function
def main():
    N = 256**3

    # probability for two RGB pixels colliding
    distribution = count()
    colliding_pairs = count_colliding_pairs(distribution)
    prob = colliding_pairs / (N*(N-1))

    print(prob)
    
    # probability for two random KxK images colliding
    K = 64**2
    prob = prob**K
    print(prob)


if __name__ == '__main__':
    main()