# Diamond Imager
Rust based web app to process images into diamond craft pictures ready to be printed.
Custom dependencies used:
- [Ditherum](https://github.com/Gieneq/Ditherum):
  - dithering & image manipulation,
  - clustering & pallette reduction,
- [DiamondsImagerGenerator](https://github.com/Gieneq/DiamondsImagerGenerator):
  - creating colors palette subset,
  - preview generation,
  - final printable PDF generation,

## Roadmap

What was done:
- [x] uploading images & assigning unique id
- [x] uploading images tests

## Endpoints idea

GET    /palette/dmc                  -> get full DMC list
POST   /image                        -> upload image
GET    /image/{id}/status            -> processing state
POST   /image/{id}/process           -> start processing with params
GET    /image/{id}/preview           -> download preview
GET    /image/{id}/result.pdf        -> download final PDF
GET    /image/{id}/bom               -> get BOM (JSON)

## Flow
1. User requests DMC colors palette
2. User select subset of palette or pass as it is,
3. User uploads image, gets **image_id** upon success,
4. User by passing **image_id** user can check:
  - status of any ongoing processing on the image,
  - start new processing
5. If processing is ready user can access:
  - download preview image,
  - download resulting PDF,
  - download Bill of Materials (BOM) as JSON

Processing params:
- image_id,
- DMC palette subset,
- paper size & margins,

BOM:
- DMC diamonds used:
  - DMC number,
  - name,
  - hex color,
- counts of diamonds