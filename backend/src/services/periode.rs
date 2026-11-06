//! Perhitungan periode laporan yang murni (tanpa basis data) sehingga mudah diuji.

use chrono::{Datelike, Duration, Months, NaiveDate};

use crate::AppError;

/// Rentang terpanjang yang dilayani laporan, agar deret tanggal tetap kecil.
pub const MAKS_HARI: i64 = 3 * 366;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kelompok {
    Hari,
    Bulan,
}

impl Kelompok {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hari => "hari",
            Self::Bulan => "bulan",
        }
    }
}

pub fn validasi(dari: NaiveDate, sampai: NaiveDate) -> Result<(), AppError> {
    if dari > sampai {
        return Err(AppError::validasi("Tanggal awal harus sebelum atau sama dengan tanggal akhir"));
    }
    if (sampai - dari).num_days() > MAKS_HARI {
        return Err(AppError::validasi("Rentang laporan maksimal 3 tahun"));
    }
    Ok(())
}

/// Tren harian untuk rentang sampai 62 hari, selebihnya bulanan.
pub fn kelompok(dari: NaiveDate, sampai: NaiveDate) -> Kelompok {
    if (sampai - dari).num_days() < 62 { Kelompok::Hari } else { Kelompok::Bulan }
}

fn akhir_bulan(tanggal: NaiveDate) -> NaiveDate {
    let awal = tanggal.with_day(1).expect("tanggal 1 selalu ada");
    awal + Months::new(1) - Duration::days(1)
}

/// Periode pembanding (KF-18f):
/// - dari tanggal 1 dan sampai pada bulan yang sama: tanggal 1 bulan sebelumnya
///   sampai hari yang sama (dipotong ke akhir bulan), misalnya "bulan ini sejauh ini";
/// - dari tanggal 1 sampai akhir suatu bulan: jumlah bulan penuh yang sama sebelumnya;
/// - selain itu: jumlah hari yang sama, berakhir sehari sebelum `dari`.
pub fn sebelumnya(dari: NaiveDate, sampai: NaiveDate) -> (NaiveDate, NaiveDate) {
    if dari.day() == 1 && (dari.year(), dari.month()) == (sampai.year(), sampai.month()) {
        let awal = dari - Months::new(1);
        // Bulan penuh dibandingkan dengan bulan penuh sebelumnya (30 Nov -> 31 Okt).
        if sampai == akhir_bulan(sampai) {
            return (awal, akhir_bulan(awal));
        }
        let hari = sampai.day().min(akhir_bulan(awal).day());
        return (awal, awal.with_day(hari).expect("hari sudah dipotong ke akhir bulan"));
    }
    if dari.day() == 1 && sampai == akhir_bulan(sampai) {
        let bulan = (sampai.year() - dari.year()) * 12 + sampai.month() as i32 - dari.month() as i32 + 1;
        let awal = dari - Months::new(bulan as u32);
        return (awal, dari - Duration::days(1));
    }
    let panjang = (sampai - dari).num_days() + 1;
    (dari - Duration::days(panjang), dari - Duration::days(1))
}

/// Rasio dalam basis poin (2537 = 25,37%), dibulatkan menjauhi nol.
/// `None` bila penyebut tidak positif (mis. penjualan bersih nol atau minus).
pub fn bps(pembilang: i64, penyebut: i64) -> Option<i64> {
    if penyebut <= 0 {
        return None;
    }
    let kali = pembilang as i128 * 10_000;
    let setengah = penyebut as i128 / 2;
    let hasil = if kali >= 0 { (kali + setengah) / penyebut as i128 } else { (kali - setengah) / penyebut as i128 };
    Some(hasil as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn bulan_penuh_dibandingkan_bulan_sebelumnya() {
        assert_eq!(sebelumnya(t(2025, 10, 1), t(2025, 10, 31)), (t(2025, 9, 1), t(2025, 9, 30)));
        assert_eq!(sebelumnya(t(2026, 1, 1), t(2026, 1, 31)), (t(2025, 12, 1), t(2025, 12, 31)));
        // Tahun kabisat: Maret 2028 dibandingkan 1-29 Februari.
        assert_eq!(sebelumnya(t(2028, 3, 1), t(2028, 3, 31)), (t(2028, 2, 1), t(2028, 2, 29)));
        assert_eq!(sebelumnya(t(2027, 3, 1), t(2027, 3, 31)), (t(2027, 2, 1), t(2027, 2, 28)));
        // Bulan 30 hari dibandingkan bulan 31 hari secara penuh, dan sebaliknya.
        assert_eq!(sebelumnya(t(2025, 11, 1), t(2025, 11, 30)), (t(2025, 10, 1), t(2025, 10, 31)));
        assert_eq!(sebelumnya(t(2025, 12, 1), t(2025, 12, 31)), (t(2025, 11, 1), t(2025, 11, 30)));
    }

    #[test]
    fn bulan_ini_sejauh_ini() {
        assert_eq!(sebelumnya(t(2025, 11, 1), t(2025, 11, 15)), (t(2025, 10, 1), t(2025, 10, 15)));
        assert_eq!(sebelumnya(t(2025, 3, 1), t(2025, 3, 30)), (t(2025, 2, 1), t(2025, 2, 28)));
    }

    #[test]
    fn beberapa_bulan_penuh() {
        assert_eq!(sebelumnya(t(2025, 4, 1), t(2025, 6, 30)), (t(2025, 1, 1), t(2025, 3, 31)));
    }

    #[test]
    fn rentang_hari_bebas() {
        assert_eq!(sebelumnya(t(2025, 10, 8), t(2025, 10, 14)), (t(2025, 10, 1), t(2025, 10, 7)));
        assert_eq!(sebelumnya(t(2025, 10, 8), t(2025, 10, 8)), (t(2025, 10, 7), t(2025, 10, 7)));
    }

    #[test]
    fn kelompok_hari_atau_bulan() {
        assert_eq!(kelompok(t(2025, 10, 1), t(2025, 10, 31)), Kelompok::Hari);
        assert_eq!(kelompok(t(2025, 1, 1), t(2025, 6, 30)), Kelompok::Bulan);
    }

    #[test]
    fn basis_poin() {
        assert_eq!(bps(2_537, 10_000), Some(2_537));
        assert_eq!(bps(1, 3), Some(3_333));
        assert_eq!(bps(2, 3), Some(6_667));
        assert_eq!(bps(-2, 3), Some(-6_667));
        assert_eq!(bps(5, 0), None);
        assert_eq!(bps(5, -10), None);
    }

    #[test]
    fn validasi_rentang() {
        assert!(validasi(t(2025, 10, 2), t(2025, 10, 1)).is_err());
        assert!(validasi(t(2020, 1, 1), t(2025, 1, 1)).is_err());
        assert!(validasi(t(2025, 10, 1), t(2025, 10, 1)).is_ok());
    }
}
