import pandas as pd
import os

os.makedirs('tests/data', exist_ok=True)

data1 = {
    'Ten cong viec': ['Xay tuong', 'Trap tuong', 'Son tuong'],
    'Don vi': ['m3', 'm2', 'm2'],
    'Khoi luong': [10.5, 200.0, 180.0]
}
pd.DataFrame(data1).to_excel('tests/data/file1.xlsx', index=False)

data2 = {
    'Ten cong viec': ['Xay tuong', 'Lat nen'],
    'Don vi': ['m3', 'm2'],
    'Khoi luong': [5.5, 50.0]
}
pd.DataFrame(data2).to_excel('tests/data/file2.xlsx', index=False)

print("Generated sample files in tests/data")
