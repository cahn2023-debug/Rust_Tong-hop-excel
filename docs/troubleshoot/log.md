# Troubleshoot Log - Missing Consolidation Files

**Sự cố**: Người dùng báo cáo một số file (ví dụ `0000. Bang tong hop...`) không được tổng hợp vào bảng cuối cùng.

## Phân tích (Council of Agents Analysis)
- **Hành vi**: File bị bỏ qua với log "No header found or data rows are empty".
- **Nguyên nhân gốc (Root Cause)**: 
    - Logic nhận diện cột "Tên công việc" (Name) trong `ExcelParser` sử dụng so sánh bằng (`==`) cho một số từ khóa chính thay vì sử dụng `contains`. 
    - Khi file có tiêu đề cột phức tạp như "VẬT TƯ, VẬT LIỆU-THIẾT BỊ...", parser không khớp được từ khóa "vật tư" vì nó không bằng chính xác chuỗi đó.
    - Thiếu một số từ khóa ngành xây dựng Việt Nam phổ biến như "Công tác", "Danh mục".

## Giải pháp triển khai
1. **ExcelParser**:
    - Chuyển đổi toàn bộ logic khớp từ khóa sang `.contains()` cho tất cả các cột để tăng tính linh hoạt.
    - Bổ sung bộ từ khóa mở rộng: `danh mục`, `công tác`, `vật liệu`, `thiết bị`.
2. **Supervisor**:
    - Cập nhật thông báo lỗi từ hardcoded sang sử dụng `analysis.reason` để hiển thị chi tiết tại sao file bị bỏ qua (ví dụ: "Header found but no data rows match filters").

## Kết quả
- Sau khi sửa, parser sẽ nhận diện đúng các cột Name có chứa từ khóa "vật tư" bất kể các ký tự đi kèm.
- Công tác báo cáo lỗi trên UI trở nên minh bạch và dễ debug hơn.

**Status**: Resolved. ✅
