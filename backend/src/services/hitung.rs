//! Perhitungan uang yang murni (tanpa basis data) sehingga mudah diuji unit.
//! Seluruh nominal dalam rupiah utuh (`i64`), tanpa bilangan pecahan.

use serde::{Deserialize, Serialize};

use crate::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetodeBayar {
    Tunai,
    Transfer,
    Qris,
}

impl MetodeBayar {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tunai => "tunai",
            Self::Transfer => "transfer",
            Self::Qris => "qris",
        }
    }
}

/// Satu baris keranjang.
#[derive(Debug, Clone, Copy)]
pub struct Baris {
    pub jumlah: i32,
    pub harga_satuan: i64,
    pub diskon_item: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rincian {
    pub subtotal_baris: Vec<i64>,
    pub subtotal: i64,
    pub diskon: i64,
    pub total: i64,
    pub bayar: i64,
    pub kembalian: i64,
}

pub fn subtotal_baris(b: &Baris) -> Result<i64, AppError> {
    if b.jumlah <= 0 {
        return Err(AppError::validasi("Jumlah barang harus lebih dari 0"));
    }
    let kotor = (b.jumlah as i64)
        .checked_mul(b.harga_satuan)
        .ok_or_else(|| AppError::validasi("Nilai transaksi terlalu besar"))?;
    if b.diskon_item < 0 || b.diskon_item > kotor {
        return Err(AppError::validasi("Diskon item tidak boleh melebihi harga barang"));
    }
    Ok(kotor - b.diskon_item)
}

/// Menghitung subtotal, total, dan kembalian. Untuk transfer/QRIS jumlah
/// bayar selalu sama dengan total sehingga kembalian 0.
pub fn hitung_transaksi(
    baris: &[Baris],
    diskon_transaksi: i64,
    metode: MetodeBayar,
    bayar: Option<i64>,
) -> Result<Rincian, AppError> {
    if baris.is_empty() {
        return Err(AppError::validasi("Keranjang masih kosong"));
    }

    let subtotal_baris = baris.iter().map(subtotal_baris).collect::<Result<Vec<_>, _>>()?;
    let subtotal: i64 = subtotal_baris.iter().sum();

    if diskon_transaksi < 0 || diskon_transaksi > subtotal {
        return Err(AppError::validasi("Diskon transaksi tidak boleh melebihi subtotal"));
    }
    let total = subtotal - diskon_transaksi;

    let bayar = match metode {
        MetodeBayar::Tunai => {
            let bayar = bayar.ok_or_else(|| AppError::validasi("Jumlah uang yang dibayar wajib diisi"))?;
            if bayar < total {
                return Err(AppError::validasi("Uang yang dibayar kurang dari total belanja"));
            }
            bayar
        }
        MetodeBayar::Transfer | MetodeBayar::Qris => total,
    };

    Ok(Rincian { subtotal_baris, subtotal, diskon: diskon_transaksi, total, bayar, kembalian: bayar - total })
}

/// Kas yang seharusnya ada di laci saat sesi ditutup (Tabel 13).
pub fn kas_seharusnya(modal_awal: i64, penjualan_tunai: i64, refund_tunai: i64) -> i64 {
    modal_awal + penjualan_tunai - refund_tunai
}

/// Selisih kas: positif berarti lebih, negatif berarti kurang.
pub fn selisih_kas(kas_fisik: i64, kas_seharusnya: i64) -> i64 {
    kas_fisik - kas_seharusnya
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baris(jumlah: i32, harga_satuan: i64) -> Baris {
        Baris { jumlah, harga_satuan, diskon_item: 0 }
    }

    #[test]
    fn tunai_menghitung_total_dan_kembalian() {
        // 2 x 85.000 + 1 x 150.000 + 1 x 45.000 = 365.000, bayar 400.000
        let r = hitung_transaksi(
            &[baris(2, 85_000), baris(1, 150_000), baris(1, 45_000)],
            0,
            MetodeBayar::Tunai,
            Some(400_000),
        )
        .unwrap();
        assert_eq!(r.subtotal_baris, vec![170_000, 150_000, 45_000]);
        assert_eq!(r.subtotal, 365_000);
        assert_eq!(r.total, 365_000);
        assert_eq!(r.kembalian, 35_000);
    }

    #[test]
    fn diskon_transaksi_10_persen_seperti_uj05() {
        // Tiga item, diskon 10% dari 300.000 = 30.000, bayar 300.000.
        let r = hitung_transaksi(
            &[baris(1, 100_000), baris(1, 120_000), baris(1, 80_000)],
            30_000,
            MetodeBayar::Tunai,
            Some(300_000),
        )
        .unwrap();
        assert_eq!(r.total, 270_000);
        assert_eq!(r.kembalian, 30_000);
    }

    #[test]
    fn diskon_item_mengurangi_subtotal_baris() {
        let b = Baris { jumlah: 3, harga_satuan: 50_000, diskon_item: 15_000 };
        assert_eq!(subtotal_baris(&b).unwrap(), 135_000);
    }

    #[test]
    fn nontunai_tidak_ada_kembalian() {
        for metode in [MetodeBayar::Transfer, MetodeBayar::Qris] {
            let r = hitung_transaksi(&[baris(1, 99_000)], 0, metode, Some(500_000)).unwrap();
            assert_eq!(r.bayar, 99_000);
            assert_eq!(r.kembalian, 0);
        }
    }

    #[test]
    fn tunai_kurang_ditolak() {
        assert!(hitung_transaksi(&[baris(1, 99_000)], 0, MetodeBayar::Tunai, Some(50_000)).is_err());
        assert!(hitung_transaksi(&[baris(1, 99_000)], 0, MetodeBayar::Tunai, None).is_err());
    }

    #[test]
    fn input_tidak_valid_ditolak() {
        assert!(hitung_transaksi(&[], 0, MetodeBayar::Qris, None).is_err());
        assert!(hitung_transaksi(&[baris(0, 10_000)], 0, MetodeBayar::Qris, None).is_err());
        assert!(hitung_transaksi(&[baris(1, 10_000)], 20_000, MetodeBayar::Qris, None).is_err());
        let diskon_berlebih = Baris { jumlah: 1, harga_satuan: 10_000, diskon_item: 10_001 };
        assert!(hitung_transaksi(&[diskon_berlebih], 0, MetodeBayar::Qris, None).is_err());
    }

    #[test]
    fn rekonsiliasi_kas_seperti_uj09() {
        // Modal 200.000, penjualan tunai 365.000, tanpa refund; kas fisik kurang 10.000.
        let seharusnya = kas_seharusnya(200_000, 365_000, 0);
        assert_eq!(seharusnya, 565_000);
        assert_eq!(selisih_kas(555_000, seharusnya), -10_000);
        assert_eq!(selisih_kas(565_000, seharusnya), 0);
        assert_eq!(kas_seharusnya(200_000, 365_000, 85_000), 480_000);
    }
}
