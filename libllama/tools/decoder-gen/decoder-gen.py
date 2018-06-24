import emitter
import argparse

parser = argparse.ArgumentParser(description='Generate a decoder given a decoder specification.')
parser.add_argument('file', type=str, nargs=1, help='decoder specification file')

args = parser.parse_args()
emitter.generate(args.file[0])