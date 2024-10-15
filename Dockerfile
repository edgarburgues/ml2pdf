# Usa una imagen base de Bun
FROM jarredsumner/bun:latest

# Establece el directorio de trabajo dentro del contenedor
WORKDIR /app

# Copia el resto de los archivos de la aplicación al contenedor
COPY . .

# Instala las dependencias de la aplicación usando Bun
RUN bun install

# Expone el puerto en el que la aplicación estará escuchando
EXPOSE 3000

# Comando para iniciar la aplicación
CMD ["bun", "index.js"]
