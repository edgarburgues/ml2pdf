# ML2PDF

<p align="center">
  <img src="assets/ml2pdf-logo.png" alt="ML2PDF Logo">
</p>

This project converts Microsoft Learn courses into PDF files by downloading and processing the content of courses and modules.

## Features

- Convert Microsoft Learn courses to PDF format.
- Download HTML content of modules and units.
- Process and convert HTML content into PDF files.
- Merge individual unit PDF files into a single PDF of the entire course.

## Usage

You can either use the default API hosted on the server or run your own API locally.

### Default usage (with the hosted API)

You can access the web interface and generate PDFs using the default hosted API:

1. Run the server with:
    ```sh
    bun index.js
    ```

2. Open a browser and visit:
    ```
    http://localhost:3000
    ```

3. Enter the Microsoft Learn course URL and follow the steps provided in the interface.

### Usage with your own locally hosted API

To use your own instance of the API:

1. Clone the repository:
    ```sh
    git clone https://github.com/edgarburgues/ml2pdf.git
    cd ml2pdf
    ```

2. Install the dependencies:
    ```sh
    bun install
    ```

3. Start the API locally with:
    ```sh
    bun index.js
    ```

4. Once the API is running on your machine, configure the web interface to use your local instance.

## Requirements

- [Bun](https://bun.sh/): Runtime environment.
- [Puppeteer](https://pptr.dev/): For handling navigation and PDF generation from HTML.
- [pdf-lib](https://pdf-lib.js.org/): To combine and process PDF files.

## Notes

- Ensure a stable internet connection while running the program.
- Temporary files generated during the process are automatically deleted after PDFs are merged.
- If an error occurs during the process, temporary files may not be deleted automatically, and you may need to delete them manually.

## Compilation with Docker

If you prefer to run the application using Docker:

1. Build the Docker image:
    ```sh
    docker build -t ml2pdf .
    ```

2. Run the container:
    ```sh
    docker run -p 3000:3000 ml2pdf
    ```

## License

This project is licensed under the MIT License. See the LICENSE file for details.
