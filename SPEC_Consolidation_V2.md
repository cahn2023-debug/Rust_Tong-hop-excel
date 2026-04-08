# SPEC: Hệ Thống Tổng Hợp Excel Chuyên Nghiệp (V2.0) - Cấu Trúc Linh Hoạt & Phân Tích Lỗi

## 1. Ngôn ngữ & Công nghệ triển khai (Tech Stack)
- **Backend (Engine)**: Rust (Tauri) - Giữ nguyên do ưu thế về tốc độ xử lý file lớn và an toàn bộ nhớ.
- **Frontend (UI)**: React + TypeScript (Vite) - Dùng TailwindCSS để thiết kế Dashboard báo cáo chuyên nghiệp.
- **Excel Library**: `calamine` (Rust) - Thư viện nhanh nhất để đọc định dạng xlsx/xlsb/xls.
- **Data Logic**: 
  - **Scoring Engine**: Thuật toán tính điểm để tìm header (đã có bản 1.0, cần nâng cấp).
  - **Anomaly Detection**: Module phân tích độ lệch (Correlation analysis) để phát hiện cột bị nhảy hoặc dữ liệu bất thường.

## 2. Các chức năng nâng cao (Feature Breakdown)

### A. Bộ máy nhận diện bảng linh hoạt (Adaptive Header Engine)
- **Mô tả**: Tự động nhận diện bảng dữ liệu dù nằm ở bất kỳ ô nào, không phụ thuộc vào hàng/cột cố định.
- **Luồng xử lý**: 
```markdown
  1. Quét toàn bộ sheet (full range) thay vì chỉ 100 hàng và 100 cột đầu.
```
  2. Dùng "Anchor matching": Tìm các cột "mỏ neo" có tính đặc trưng cao (STT, Khối lượng).
  3. Tính toán "Table Geometry": Xác định vùng bao quanh bảng dữ liệu thực.
- **Edge Cases**: File có nhiều bảng trong 1 sheet; File có header nằm dọc (ít gặp nhưng cần tính đến).

### B. Module Phân Tích & Báo Cáo Độ Lệch (Analysis Module)
- **Mô tả**: Tính năng "Analysis" để tổng hợp các bảng lệch so với tổng thể.
- **Tính năng con**:
  - **Structure Audit**: So sánh cấu trúc file hiện tại với "Cấu trúc chuẩn" (do user định nghĩa hoặc AI tự học từ đa số các file). Báo cáo nếu cột bị đổi chỗ.
  - **Unit Sync**: Phát hiện các đơn vị tính bị viết sai (m3 vs m.3 vs khối...).
  - **Outlier Detection**: Cảnh báo nếu khối lượng một hàng tăng đột biến (ví dụ gấp 100 lần trung bình) - dấu hiệu nhập sai.
- **Output**: Một bảng Dashboard "Quality Report" hiển thị danh sách các file/sheet cần review thủ công.

### C. Giao diện Dashboard Phối Hợp (Integrated Dashboard)
- **Mô tả**: Hiển thị tiến độ và kết quả phân tích theo thời gian thực.
- **Components**:
  - **Total Progress**: % hoàn thành.
  - **Success vs. Warnings**: Biểu đồ tròn hiển thị số file OK và số file bị "Lệch".
  - **Log Detail**: Click vào file bị lỗi để xem chi tiết "Tại sao tôi cho rằng nó bị lệch" (ví dụ: "Cột Khối lượng bị đẩy sang phải 2 ô").

## 3. Các giải pháp tối ưu (Optimization & Scalability)
- **Parallel Processing**: Tận dụng `rayon` để xử lý hàng trăm file cùng lúc (Đã có).
- **Cache Layer**: Dùng SQLite để lưu metadata và kết quả phân tích, tránh quét lại các file cũ nếu không thay đổi (Đã có).
- **Flexibility**: Cho phép User "Dạy" app bằng cách kéo thả vị trí cột nếu AI nhận diện sai (Human-in-the-loop).

---

### 🛠️ CÂU HỎI TƯ VẤN (Socratic Method)
Sếp ơi, để em hoàn thiện bản thiết kế này, sếp cho em biết thêm:
1. Sếp có một file mẫu nào được coi là "Chuẩn" không? Hay em sẽ tự động lấy file có cấu trúc phổ biến nhất làm chuẩn?
2. Ngoài "Khối lượng", sếp có cần tổng hợp thêm các cột tiền (Đơn giá, Thành tiền) không?
3. Sếp muốn báo cáo "Analysis" hiển thị ngay trên UI hay xuất ra một file Excel "Audit_Report.xlsx" riêng?

**Sếp thấy bản thiết kế hệ thống này đã ổn chưa? Có muốn điều chỉnh gì trước khi em bắt đầu code không ạ?**
