#DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf
#Phi-3.5-mini-instruct-IQ2_M.gguf
#tinyllama-1.1b-chat-v1.0.Q4_0.gguf


sudo apt update
sudo apt upgrade -y
sudo apt install -y build-essential cmake git curl wget libcurl4-openssl-dev pkg-config gcc-9 g++-9

sudo apt install software-properties-common
sudo add-apt-repository ppa:deadsnakes/ppa
sudo apt install python3.6 python3.6-venv python3.6-dev
wget https://download.pytorch.org/whl/cu110/torch-1.7.1%2Bcu110-cp36-cp36m-linux_x86_64.whl
wget https://download.pytorch.org/whl/cu110/torchvision-0.8.2%2Bcu110-cp36-cp36m-linux_x86_64.whl


sudo ubuntu-drivers autoinstall

sleep 10

CUDA_RUN_FILE="cuda_11.0.3_450.51.06_linux.run"
wget https://developer.download.nvidia.com/compute/cuda/11.0.3/local_installers/$CUDA_RUN_FILE
chmod +x $CUDA_RUN_FILE
sudo ./$CUDA_RUN_FILE --silent --toolkit

echo 'export PATH=/usr/local/cuda-11.0/bin:$PATH' >> ~/.bashrc
echo 'export LD_LIBRARY_PATH=/usr/local/cuda-11.0/lib64:$LD_LIBRARY_PATH' >> ~/.bashrc
source ~/.bashrc

nvidia-smi
nvcc --version

python3 -m venv llama_venv
source llama_venv/bin/activate
pip install --upgrade pip
pip install torch==1.12.1+cu110 torchvision==0.13.1+cu110 torchaudio==0.12.1 --extra-index-url https://download.pytorch.org/whl/cu110

cd ~
git clone https://github.com/ggerganov/llama.cpp.git
cd llama.cpp
rm -rf build
mkdir build
cd build


# EDIT FILE:
#ifdef __CUDACC__
  #ifndef __builtin_assume
    #define __builtin_assume(x)  // empty: ignore it for CUDA compiler
  #endif
#endif
# ggml/src/ggml-cuda/fattn-common.
#
cmake -DGGML_CUDA=ON -DCMAKE_CUDA_ARCHITECTURES=35 ..
make -j$(nproc)

OR
wget https://github.com/ggml-org/llama.cpp/releases/download/b6148/llama-b6148-bin-ubuntu-x64.zip


wget https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_0.gguf?download=true
