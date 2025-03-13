from flask import Flask, send_file, request, abort
import redis
from io import BytesIO
from PIL import Image
import os

app = Flask(__name__)

# Configuração do Redis
redis_client = redis.Redis(host='redis', port=6379, db=0)

# Pasta onde as imagens estão armazenadas
IMAGE_FOLDER = 'images'

@app.route('/image/<directory>/<filename>')
def get_image(directory, filename):
    filename = directory + "/" + filename

    # Verifica se a imagem está no cache do Redis
    cached_image = redis_client.get(filename)

    if cached_image:
        # Se a imagem estiver no cache, retorna ela diretamente
        return send_file(BytesIO(cached_image), mimetype='image/jpeg', as_attachment=True, download_name='logo.jpeg')

    # Se não estiver no cache, tenta carregar a imagem do disco
    image_path = os.path.join(IMAGE_FOLDER, filename)
    if not os.path.isfile(image_path):
        abort(404)  # Retorna 404 se a imagem não existir
    
    # Abre a imagem usando Pillow
    img = Image.open(image_path)
    
    # Converte a imagem para bytes
    img_byte_arr = BytesIO()
    img.save(img_byte_arr, format='JPEG')
    img_byte_arr = img_byte_arr.getvalue()
    
    # Armazena a imagem no cache do Redis
    redis_client.set(filename, img_byte_arr)
    
    # Retorna a imagem
    return send_file(BytesIO(img_byte_arr), mimetype='image/jpeg', as_attachment=True, download_name='logo.jpeg')

if __name__ == '__main__':
    app.run(host="0.0.0.0", debug=True)
