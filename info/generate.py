import matplotlib.pyplot as plt
import pandas as pd

# Data
data = {
    "Size": [10, 10, 10, 100, 100, 100, 1000, 1000, 1000, 10000, 10000, 10000, 100000, 100000, 100000],
    "RDB time (µs)": [36, 36, 36, 37, 37, 37, 41, 37, 39, 60, 40, 42, 232, 59, 77],
    "Sled time (µs)": [1, 0.1, 0.2, 16, 1, 3, 206, 18, 34, 3000, 287, 534, 40000, 4000, 7000],
    "Details": ["Regular", "10%", "20%", "Regular", "10%", "20%", "Regular", "10%", "20%", "Regular", "10%", "20%", "Regular", "10%", "20%"]
}
df = pd.DataFrame(data)

# Colors for different Details
colors = {
    "Regular": "blue",
    "10%": "green",
    "20%": "red"
}

# Plotting
fig, ax = plt.subplots(figsize=(18, 10))
num_details = df["Details"].nunique()
bar_width = 0.2
positions = list(range(df["Size"].nunique()))

for index, (detail, color) in enumerate(colors.items()):
    sub_df = df[df["Details"] == detail]
    rdb_positions = [p + index * bar_width for p in positions]
    sled_positions = [p + bar_width / 2 + index * bar_width for p in positions]
    rdb_bars = ax.bar(rdb_positions, sub_df["RDB time (µs)"], color=color, width=bar_width, edgecolor='gray', label=f"RDB {detail}")
    sled_bars = ax.bar(sled_positions, sub_df["Sled time (µs)"], color=color, width=bar_width, edgecolor='gray', alpha=0.6, label=f"Sled {detail}")

    for bars in [rdb_bars, sled_bars]:
        for bar in bars:
            yval = bar.get_height()
            ax.text(bar.get_x() + bar.get_width()/2, yval, round(yval, 2), ha='center', va='bottom', fontsize=8, fontweight='bold', color='black')

ax.set_xticks([p + bar_width for p in positions])
ax.set_xticklabels(df["Size"].unique())
ax.set_xlabel("Size")
ax.set_ylabel("Time (µs)")
ax.set_title("Comparison of RDB and Sled Time by Details")
ax.legend(loc="upper left", bbox_to_anchor=(1,1), title="Details")
ax.set_yscale("log")
plt.tight_layout()
plt.show()
