# Diamond Imager

<p align="center"> 🚧 </p>
<p align="center"> **This project is currently in active development.** </p>
<p align="center"> Expect breaking changes and unfinished features. </p>
<p align="center"> 🚧 </p>

Rust & Axum web app to process images into Diamond Painting projects ready to be printed.

<p align="center">
  <img alt="Example printed output" src="res/dithering_result.png">
</p>
<p align="center">
  <em> Example of printed output </em>
</p>

## Flow idea
1. **User uploads image**:
  - Image is stored as-is
  - User gets a UUID
  - Image is accessible for a limited time
2. **User request one of DMC palettes**:
  - Full DMC palette
  - Subset of colors most representative of the uploaded image
3. **User loops**:
  - Optionally modify the palette manually or re-extract subset
  - Adjust color manipulation parameters
  - Crop/rotate the image
  - Request preview
  - Generate final PDF

## Endpoints
| Method | Route                       | Effect | Implemented |
|--------|-----------------------------|---|---|
| POST   | /api/image                  | Upload image obtain UUID | Y |
| GET    | /api/image/{uuid}           | Get image metadata (e.g., upload time, resolution) | Y |
| DELETE | /api/image/{uuid}           | Delete uploaded image manually | Y |
| GET    | /api/palette/dmc            | Get full DMC list | Y |
| POST   | /api/palette/extract/{uuid} | Start palette extraction from image if not busy | n |
| GET    | /api/palette/extract/{uuid} | Get palette extraction from image if ready | n |
| POST   | /api/preview/{uuid}         | Start generating preview image PNG if not busy | n |
| GET    | /api/preview/{uuid}         | Download preview image PNG if ready | n |
| POST   | /api/pdf/{uuid}             | Start generating printable PDF if not busy | n |
| GET    | /api/pdf/{uuid}             | 	Download the generated PDF if ready | n |
| GET    | /api/processing/{uuid}      | 	Check processing status (useful for async steps) | n |
| POST   | /api/image/{uuid}/transform | Crop/rotate/adjust brightness/contrast | ? |

**Note**: palette will be attached in query parameters while requesting preview or PDF.

## Todo
- [x] Proof of concept
- [x] Basic image upload with UUID return
- [x] Store images temporarily with expiration
- [x] DMC full palette support
- [x] Automatic DMC palette extraction from image
- [ ] Manual palette editing interface
- [ ] Image preview generation
- [ ] Image manipulation:
  - [ ] Crop
  - [ ] Rotate
  - [ ] Brightness/contrast adjustments
- [ ] Debounced real-time preview updates (client or server)
- [x] PDF generation with DMC color grid
- [ ] API route documentation
- [ ] Client-side frontend (Wanna try WASM)
- [ ] Style output for printing (grid size, margins, legend)
- [ ] Image cleanup job (periodic deletion of expired images)
- [ ] Consider user accounts (Diesel ORM & Postgress)
- [ ] Deployment setup (Docker, hosting config)
- [ ] Add tests (unit & integration)
- [ ] CI pipeline (formatting, clippy, tests)