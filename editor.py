import struct
import zlib
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
        self.chunks = []
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


    # helper function to read a chunk
    # TODO: fix this
    def read_chunk(self, f):
        chunk_length, chunk_type = struct.unpack(">I4s", f.read(8))
        print(f"[*] Chunk type: {chunk_type}, chunk length {chunk_length}")
        chunk_data = f.read(chunk_length)
        expected_crc = zlib.crc32(chunk_type + chunk_data) & 0xffffffff
        actual_crc, = struct.unpack(">I", f.read(4))
        # check the crc for data corruption
        assert expected_crc == actual_crc

        return chunk_type, chunk_data


    # parse the png and save chunks in separate buffers, consolidate pixels to one buffer
    # TODO: follow this guide https://pyokagan.name/blog/2019-10-14-png/ 
    def parse_png(self):
        print("[*] Parsing png chunks")

        png_signature = b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"
        with open(self.image_path, "rb") as f:
            actual_signature = f.read(8)
            assert actual_signature == png_signature

            # parse all chunks out of the file
            while True:
                chunk_type, chunk_data = self.read_chunk(f)
                self.chunks.append([chunk_type, chunk_data])
                if chunk_type == b"IEND":
                    break

            print([chunk[0] for chunk in self.chunks])

            
# main function
def main():
    # editor functionality
    editor = Editor("original.png")
    editor.parse_png()
    #editor.greyscale()


if __name__ == '__main__':
    main()