# 📊 BÁO CÁO DỰ ÁN: Excel Engine Ultra

## 🎯 App này làm gì?
Excel Engine Ultra là một công cụ xử lý dữ liệu Excel (ETL) hiệu năng cao dành cho máy tính để bàn. Nó cho phép quét hàng ngàn file Excel, tự động nhận diện cấu trúc bảng (Header) và tổng hợp dữ liệu vào một kho lưu trữ tập trung với tốc độ cực nhanh nhờ sức mạnh của ngôn ngữ Rust.

## 📁 Cấu trúc chính
```text
.
├── excel-ui/               # Toàn bộ mã nguồn Frontend & Tauri
│   ├── src/                # Giao diện người dùng (TS/Vite)
│   ├── src-tauri/          # Backend Rust (Core Engine)
│   │   ├── src/
│   │   │   ├── aggregator/ # Logic tổng hợp dữ liệu
│   │   │   ├── cache/      # Tối ưu hóa với SQLite
│   │   │   ├── parser/     # Bộ máy đọc file Excel (Calamine)
│   │   │   └── pipeline/   # Hệ thống Actor & Giám sát (Supervisor)
│   └── public/             # Tài nguyên tĩnh & Icons
├── docs/                   # Tài liệu hướng dẫn & Nhật ký
└── tests/                  # Kiểm thử tự động
```

## 🛠️ Công nghệ sử dụng
| Thành phần | Công nghệ | Ưu điểm |
|------------|-----------|---------|
| **Core Engine** | Rust (Tauri 2.0) | Xử lý đa luồng, an toàn bộ nhớ, tốc độ cao. |
| **Giao diện** | Vite + TS + Vanilla CSS | Nhẹ, hiện đại, tối ưu hiệu suất render. |
| **Excel Parser**| Calamine | Thư viện Rust nhanh nhất để giải mã .xlsx/.xlsb. |
| **Database/Cache**| SQLite | Lưu trữ metadata file để tránh Re-scan, tăng tốc 10x. |
| **Concurrency** | Rayon + Actor Model | Xử lý song song không gây nghẽn UI. |

## 🚀 Trạng thái hiện tại (v1.3.0)
- **Build:** ✅ Thành công bản Production MSI.
- **Sync:** ✅ Đã đẩy lên GitHub thành công.
- **Tính năng nổi bật:**
  - Nhận diện Header linh hoạt (Adaptive Header).
  - Tự động sửa lỗi sai lệch cột (Anomaly Detection).
  - Highlight file nguồn trực tiếp từ bảng tổng hợp.

## 📍 Task tiếp theo (Roadmap)
- [ ] Tích hợp AI (LLM) để phân tích nội dung dữ liệu sâu hơn.
- [ ] Hỗ trợ xuất dữ liệu ra định dạng SQL/PowerBI.
- [ ] Thêm chế độ Dark/Light mode tự động theo hệ điều hành.

## 📝 Các file quan trọng cần biết
| File | Chức năng |
|------|-----------|
| `excel-ui/src-tauri/src/main.rs` | Điểm vào của ứng dụng & định nghĩa Command. |
| `excel-ui/src-tauri/src/pipeline/supervisor.rs` | "Bộ não" điều phối luồng xử lý dữ liệu. |
| `excel-ui/src/ui.ts` | Logic điều khiển tương tác người dùng trên giao diện. |

---
*Báo cáo được tạo bởi Antigravity AI vào lúc 2026-04-08.*
