from PIL import Image

'''
- This script aims to demonstrate editor functionality
'''


# class for all functionality related to the editor
class Editor:
    def __init__(self, image_path):
        self.image_path = image_path
        self.r_coeff = 30
        self.g_coeff = 58
        self.b_coeff = 11
        self.r = []
        self.g = []
        self.b = []
        self.greyscale_pixels = []
        self.target = "greyscale.png"

    # write the pixels to output/greyscale.txt as public input to the circuit (should be extracted from edited image)
    def write_to_output(self):
        with open("output/greyscale.txt", "w") as f:
            for i in range(len(self.greyscale_pixels)):
                if i == len(self.greyscale_pixels) - 1:
                    f.write(str(self.greyscale_pixels[i]))
                
                else:
                    f.write(str(self.greyscale_pixels[i]) + "\n")

        f.close()

    # write the pixel values to a csv for debugging
    def write_to_csv(self):
        with open("output/python_pixels.csv", "w") as f:
            assert len(self.r) == len(self.g)
            assert len(self.r) == len(self.b)

            for i in range(len(self.r)):
                f.write(str(self.r[i]) + "," + str(self.g[i]) + "," + str(self.b[i]) + "\n")

        f.close()

    # greyscale the image and save as a new jpg
    def greyscale(self):
        print(f"[*] Greyscaling image: {self.image_path}")
        image = Image.open(self.image_path).convert("RGB")
        width, height = image.size
        pixels = image.load()

        # greyscale image
        grey_img = Image.new("L", (width, height)) # create a new image to write to - greyscale
        grey_pixels = grey_img.load()

        # loop over all pixels - going along and then up the rows
        for y in range(height):
            for x in range(width):
                r_val, g_val, b_val = image.getpixel((x, y))
                self.r.append(r_val)
                self.g.append(g_val)
                self.b.append(b_val)

                # greyscale
                grey_val = (self.r_coeff * r_val + self.g_coeff * g_val + self.b_coeff * b_val) // 100
                self.greyscale_pixels.append(grey_val)
                grey_pixels[x, y] = grey_val

        print(f"[*] Saving greyscaled image as: {self.target}")
        grey_img.save(self.target)


# main function
def main():
    # editor functionality
    editor = Editor("original.png")
    editor.greyscale()
    editor.write_to_output()
    editor.write_to_csv()


if __name__ == '__main__':
    main()