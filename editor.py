from PIL import Image

    
# read the bytes of the png into an array
def edit_img(image):
    img = Image.open(image).convert("RGB")
    width, height = img.size
    pixels = img.load()
    grey_img = Image.new("L", (width, height)) # create a new image to write to - greyscale
    grey_pixels = grey_img.load()
    
    # iterate over the pixels and extract the 3 bytes for color channels (R, G, B)
    for y in range(height):
        for x in range(width):
            r, g, b = pixels[x, y]
            grey_pixel = (30*r + 58*g + 11*b) // 100
            grey_pixels[x, y] = grey_pixel

    grey_img.save("greyscale.png")


# main function
def main():
    image = "image.png" # relative path
    edit_img(image)


if __name__ == '__main__':
    main()