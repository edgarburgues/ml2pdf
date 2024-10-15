import express from "express";
import https from "https";
import puppeteer from "puppeteer";
import path from "path";
import { fileURLToPath } from "url";
import { PDFDocument } from "pdf-lib";
import { Server } from "socket.io";
import http from "http";
import fs from "fs";
import os from "os";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const app = express();
const port = 3000;

const server = http.createServer(app);
const io = new Server(server);

app.use(express.json());
app.use(express.static(path.join(__dirname, "static")));

// Servir el archivo HTML
app.get("/", (req, res) => {
    res.sendFile(path.join(__dirname, "views", "index.html"));
});

const BATCH_SIZE = 50; // Número de PDFs a combinar en cada lote

async function combinePDFsInBatches(pdfFiles, log) {
    let currentBatch = [];
    let intermediateFiles = [];

    for (let i = 0; i < pdfFiles.length; i++) {
        currentBatch.push(pdfFiles[i]);

        if (currentBatch.length === BATCH_SIZE || i === pdfFiles.length - 1) {
            log(`Combinando un lote de ${currentBatch.length} PDFs...`);
            const batchMergedPdf = await PDFDocument.create();

            for (const pdfFile of currentBatch) {
                const existingPdfBytes = fs.readFileSync(pdfFile);
                const singlePdf = await PDFDocument.load(existingPdfBytes);
                const copiedPages = await batchMergedPdf.copyPages(singlePdf, singlePdf.getPageIndices());
                copiedPages.forEach(page => batchMergedPdf.addPage(page));
            }

            const batchPdfPath = path.join(os.tmpdir(), `batch-${intermediateFiles.length + 1}.pdf`);
            fs.writeFileSync(batchPdfPath, await batchMergedPdf.save());
            intermediateFiles.push(batchPdfPath);
            currentBatch = [];
        }
    }

    log('Combinando todos los lotes intermedios...');
    const finalMergedPdf = await PDFDocument.create();

    for (const intermediateFile of intermediateFiles) {
        const existingPdfBytes = fs.readFileSync(intermediateFile);
        const singlePdf = await PDFDocument.load(existingPdfBytes);
        const copiedPages = await finalMergedPdf.copyPages(singlePdf, singlePdf.getPageIndices());
        copiedPages.forEach(page => finalMergedPdf.addPage(page));
    }

    return finalMergedPdf;
}

io.on('connection', (socket) => {
    socket.on('generate-pdf', async (data) => {
        const { url } = data;

        try {
            const log = (message) => {
                console.log(message);
                socket.emit('log', message);
            };

            log(`Iniciando proceso para la URL: ${url}`);

            // Crear un directorio temporal para almacenar los PDFs individuales
            const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'pdf-temp-'));
            const pdfFiles = [];

            log(`Consultando API para obtener los módulos...`);
            const apiData = await new Promise((resolve, reject) => {
                https.get(`https://ml2pdf-web-api.azurewebsites.net/api?url=${url}`, (resp) => {
                    let data = '';

                    resp.on('data', (chunk) => {
                        data += chunk;
                    });

                    resp.on('end', () => {
                        resolve(JSON.parse(data));
                    });

                }).on("error", (err) => {
                    reject(err);
                });
            });
            log(`Datos obtenidos de la API.`);

            log(`Iniciando Puppeteer...`);
            const browser = await puppeteer.launch({
                headless: true,
                args: ['--no-sandbox', '--disable-setuid-sandbox']
            });
            const page = await browser.newPage();

            await page.setViewport({
                width: 1280,
                height: 1024,
                deviceScaleFactor: 1,
            });

            const totalModules = Object.values(apiData).reduce((acc, val) => acc + val.length, 0);
            let completedModules = 0;

            for (let [key, value] of Object.entries(apiData)) {
                log(`Procesando módulo: ${key}`);
                for (let item of value) {
                    log(`- Visitando página: ${item.url}`);
                    await page.goto(item.url, { waitUntil: 'networkidle0' });

                    const pdfPath = path.join(tempDir, `page-${completedModules + 1}.pdf`);
                    await page.pdf({
                        path: pdfPath,
                        format: 'A4',
                        printBackground: true,
                        preferCSSPageSize: true,
                        omitBackground: false,
                    });

                    pdfFiles.push(pdfPath);
                    completedModules++;
                    log(`Progreso: ${completedModules} de ${totalModules}`);
                }
            }

            log(`Cerrando Puppeteer...`);
            await browser.close();

            log(`Combinando los PDFs en lotes...`);
            const finalMergedPdf = await combinePDFsInBatches(pdfFiles, log);

            const finalPdfBuffer = await finalMergedPdf.save();

            // Guardar el archivo en la carpeta en la que se ejecuta el servidor
            const fileName = `result-${Date.now()}.pdf`;
            const finalFilePath = path.join(__dirname, fileName);
            fs.writeFileSync(finalFilePath, finalPdfBuffer);

            log(`PDF generado y guardado en ${finalFilePath}`);
            socket.emit('done', fileName);

            log(`Proceso completado para la URL: ${url}`);

            // Limpiar archivos temporales
            pdfFiles.forEach(file => fs.unlinkSync(file));

        } catch (error) {
            console.error(`Error durante el proceso: ${error.message}`);
            socket.emit('error', error.message);
        }
    });
});

// Iniciar el servidor
server.listen(port, () => {
    console.log(`App running at http://localhost:${port}`);
});
