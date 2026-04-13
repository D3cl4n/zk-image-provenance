from PIL import Image

'''
- This script aims to demonstrate editor functionality
- TODO: rewrite this to read image bytes directly and only greyscale pixels from IDATA chunk
    - need to save all bytes 
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


    # greyscale the image and save as a new jpg
    def greyscale(self): # TODO: change color type in IHDR chunk to be 0 for greyscale
        print(f"[*] Greyscaling image: {self.image_path}")
        image = Image.open(self.image_path).convert("RGB")
        width, height = image.size
        pixels = image.load()

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
                pixels[x, y] = (grey_val, grey_val, grey_val)

        print(f"[*] Saving greyscaled image as: {self.target}")
        image.save(self.target)


    # parse the png and save chunks in separate buffers, consolidate pixels to one buffer
    # TODO: follow this guide https://pyokagan.name/blog/2019-10-14-png/ 
    def parse_png(self):
        print("[*] Parsing png chunks")

        png_signature = b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"
        with open(self.image_path, "rb") as f:
            png_data = f.read()
            if not png_data.startswith(png_signature):
                print("[!] File is not a PNG")
                exit(-1) 

        # locate the IDAT chunk
            

# main function
def main():
    # editor functionality
    editor = Editor("original.png")
    editor.greyscale()


if __name__ == '__main__':
    main()