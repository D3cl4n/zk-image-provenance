from PIL import Image


# class for PNG parsing functionality
class PNGUtils:
    def __init__(self, image_path):
        self.image = image_path
        self.png_signature = b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"
        # {chunk_type : (length, chunk_type, data, crc)}
        self.chunks = {} 

    # helper function to read a single chunk from the image
    def read_single_chunk(self, f):
        pass

    # read all chunks from the image
    def read_all_chunks(self):
        pass

    # flip the color type field of the IHDR chunk to 0 to indicate a greyscaled image
    def flip_color_type(self):
        pass

    # reassemble the png given the dictionary of chunks
    def reassemble_png(self):
        pass


# class for the editor functionality
class EditorUtils:
    def __init__(self, image_path):
        self.image = image_path
        self.target = "greyscale.png"
        # coefficients are rounded to the nearest integer to work in a circuit
        self.r_coeff = 30
        self.g_coeff = 58
        self.b_coeff = 11
        self.png_utils = PNGUtils(self.image)

    # greyscale the pixels in the IDAT chunk(s) - after IDAT chunks are aggregated
    def greyscale(self):
        pass


# main function
def main():
    pass


if __name__ == '__main__':
    main()