# 📚 Readatabase (readb)
![crates.io](https://img.shields.io/crates/v/readb.svg)
[![Rust CI](https://github.com/Basis-Health/readb/actions/workflows/rust.yml/badge.svg)](https://github.com/Basis-Health/readb/actions/workflows/rust.yml)

## 🚨 **Update Alert**: BREAKING CHANGES in 0.4.0
- 🛠 Refactored database code structure. You must now use `use readb::Database;`.
- ✅ The constructor no longer needs `unwrap()`. An `.unwrap()` method remains for backward compatibility.
   - 🔁 Instead, we've introduced the `create_path` attribute in `DatabaseSettings`. This could be a breaking change if you aren’t using the `Default`.

---

🔍 **"Don't reinvent the wheel."** Despite this wisdom, here we are with **readb**: a fresh, streamlined embedded key-value database crafted in pure Rust.

Balancing simplicity akin to sled, readb boasts outstanding read performance. Primarily focused on reads, it also caters to writes and deletes. It remains lightweight with minimal dependencies and under 1KB size.

## 🌟 Features
- **Custom Cache**: Choose from an array of caching strategies (some still under development).
- **Lock-Free Reads**: Optimized for concurrent access.
- **Remote Cloning**: Sync your data with ease.
- **Transactions**: Secure and efficient data modifications.

## 🚀 Speed Secrets of readb
readb thrives on being read-centric. Assuming data remains largely static, we strike a balance between memory efficiency and speed by leveraging disk and memory effectively.

Data management includes:
- **link**: Associate without duplicating.
- **put**: Append data linearly for quick access.

readb's linear approach stands out from the norm, ensuring swift operations.

## 🎯 Ideal Users?
If you predominantly deal with read requests and yearn for a speedy, efficient local solution, readb awaits. While it excels in write speeds, it does so with minor compromises on compression.

## 📊 Benchmarks

**Write Benchmark**:

| Operations | `readb` max time | `sled` max time |
|------------|------------------|-----------------|
| 1,000      | 147.02 µs        | 332.65 µs       |
| 1,000,000  | 238.48 ms        | 485.63 ms       |

**Read Benchmark**:

| Benchmark type                                           | time readb   | time sled  | time redb  |
|----------------------------------------------------------|------------|------------|------------|
| Retrieve 1000 items                                      | 50.30      | 79.25      | 97.95      |
| Retrieve 1000 items (10 percent)                         | 49.05      | 17.84      | 22.39      |
| Retrieve 1000 items (20 percent with repetitions)        | 49.16      | 33.92      | 43.17      |
| Retrieve 10000 items                                     | 67.56      | 1,256.75   | 1,225.68   |
| Retrieve 10000 items (10 percent)                        | 55.29      | 311.57     | 357.98     |
| Retrieve 10000 items (20 percent with repetitions)       | 61.13      | 541.79     | 692.16     |
| Retrieve 100000 items                                    | 209.59     | 36,977     | 52,558     |
| Retrieve 100000 items (10 percent)                       | 86.25      | 3,465.1    | 5,090.9    |
| Retrieve 100000 items (20 percent with repetitions)      | 124.29     | 6,831.8    | 10,151     |

**Visuals**: ![graph](./info/img.png)

## 🛠 Getting Started

### 1. **Database Setup**
The `Database` struct manages indexing, caching, and data loading and requires two generics:
- `C`: Cache mechanism (implement the `Cache` trait).
- `L`: Data loader (implement the `Loader` trait).

### 2. **Operations**:
From fetching keys to maintaining data consistency, we've got you covered.

### 3. **Advanced Features**:
From data writes to efficient garbage management, elevate your database game.

## 🛣 Roadmap to 1.0
With a stabilized API, our focus shifts to refining caching and enhancing documentation. As we inch towards our 1.0 release, backward compatibility remains paramount.

## 📝 License
Licensed under the Apache License, Version 2.0. Dive into the NOTICE file for more details.