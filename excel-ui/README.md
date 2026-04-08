# Excel Engine Ultra (Tauri + Vanilla TS)

Phần mềm quét và xử lý file Excel, được xây dựng bằng Tauri và Vanilla TypeScript.

## Tính năng
- Scan thư mục tìm kiếm file .xlsx, .xls, .xlsm...
- Giao diện hiện đại, tối giản.
- Menu chuột phải: Mở file trực tiếp hoặc mở thư mục chứa file.
- Hỗ trợ phím tắt và tương tác mượt mà.

## Built with
- **Frontend**: Vite, TypeScript, Vanilla CSS.
- **Backend**: Rust (Tauri 2.0).

## Build
Để build bản cài đặt (.msi) cho Windows:
```bash
npm run tauri build
```
File installer sẽ nằm trong `src-tauri/target/release/bundle/msi/`.

## Author
Thanh Bui
