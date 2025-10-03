from datasets import load_dataset

def main():
    dataset = load_dataset('jordiae/exebench', split='test_synth')
    print(dataset)

if __name__ == "__main__":
    main()
