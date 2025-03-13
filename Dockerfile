# Use uma imagem base do Ubuntu
FROM ubuntu:latest

# Atualiza o índice de pacotes e instala os pacotes necessários
RUN apt-get update && \
    apt-get install -y \
    python3-flask \
    python3-redis \
    python3-pil \
    redis-server \
    git && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Define o diretório de trabalho
WORKDIR /app

COPY . .

# Expõe a porta que o Flask vai usar
EXPOSE 5000

# Comando para rodar a aplicação
CMD ["python3", "main.py"]
