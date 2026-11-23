//! A data e a hora locais sem programa de fora: o `date` não existe no Windows e o do macOS não
//! aceita `-d` (o rodapé caía no UTC e os recentes ficavam sem data). O fuso vem do sistema; o
//! calendário é conta.

/// Minutos a leste de UTC agora (Brasília: -180). Sem fuso conhecido, 0.
pub fn deslocamento_minutos() -> i64 {
    #[cfg(unix)]
    {
        let agora = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as libc::time_t;
        // SAFETY: `localtime_r` só escreve na `tm` que passamos.
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        if !unsafe { libc::localtime_r(&agora, &mut tm) }.is_null() {
            return tm.tm_gmtoff as i64 / 60;
        }
        0
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::SYSTEMTIME;
        use windows_sys::Win32::System::SystemInformation::{GetLocalTime, GetSystemTime};
        let minutos = |t: &SYSTEMTIME| dias_desde_1970(t.wYear as i64, t.wMonth as i64, t.wDay as i64) * 1440 + t.wHour as i64 * 60 + t.wMinute as i64;
        // SAFETY: as duas só preenchem a estrutura que passamos.
        let (mut local, mut utc): (SYSTEMTIME, SYSTEMTIME) = unsafe { (std::mem::zeroed(), std::mem::zeroed()) };
        unsafe { GetLocalTime(&mut local); GetSystemTime(&mut utc); }
        minutos(&local) - minutos(&utc)
    }
    #[cfg(not(any(unix, windows)))]
    { 0 }
}

/// Dias desde 1/1/1970 da data civil (algoritmo de Howard Hinnant).
fn dias_desde_1970(ano: i64, mes: i64, dia: i64) -> i64 {
    let a = if mes <= 2 { ano - 1 } else { ano };
    let era = a.div_euclid(400);
    let ano_da_era = a - era * 400;
    let dia_do_ano = (153 * (mes + if mes > 2 { -3 } else { 9 }) + 2) / 5 + dia - 1;
    let dia_da_era = ano_da_era * 365 + ano_da_era / 4 - ano_da_era / 100 + dia_do_ano;
    era * 146_097 + dia_da_era - 719_468
}

/// A data civil (ano, mês, dia) de um número de dias desde 1/1/1970.
fn data_civil(dias: i64) -> (i64, i64, i64) {
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let dia_da_era = z - era * 146_097;
    let ano_da_era = (dia_da_era - dia_da_era / 1460 + dia_da_era / 36_524 - dia_da_era / 146_096) / 365;
    let dia_do_ano = dia_da_era - (365 * ano_da_era + ano_da_era / 4 - ano_da_era / 100);
    let mp = (5 * dia_do_ano + 2) / 153;
    let dia = dia_do_ano - (153 * mp + 2) / 5 + 1;
    let mes = if mp < 10 { mp + 3 } else { mp - 9 };
    (ano_da_era + era * 400 + i64::from(mes <= 2), mes, dia)
}

/// `("dd/mm/aaaa", "HH:MM")` de um instante (segundos UTC) com o deslocamento dado.
fn formatar_com(segundos: i64, minutos_leste: i64) -> (String, String) {
    let local = segundos + minutos_leste * 60;
    let (dias, resto) = (local.div_euclid(86_400), local.rem_euclid(86_400));
    let (a, m, d) = data_civil(dias);
    (format!("{d:02}/{m:02}/{a:04}"), format!("{:02}:{:02}", resto / 3600, resto % 3600 / 60))
}

/// `("dd/mm/aaaa", "HH:MM")` locais de um instante em segundos UTC.
// shortcut: usa o fuso de agora (uma data de antes da mudança de horário de verão pode sair 1 h diferente); usar o fuso da data se alguém notar.
pub fn formatar(segundos: i64) -> (String, String) {
    formatar_com(segundos, deslocamento_minutos())
}

/// `("dd/mm/aaaa", "HH:MM")` de agora, na hora local.
pub fn agora() -> (String, String) {
    formatar(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn conta_o_calendario_e_o_fuso() {
        assert_eq!(formatar_com(0, 0), ("01/01/1970".into(), "00:00".into()));
        // 23/11/2026 02:30 UTC = 22/11 23:30 em Brasília (o dia volta).
        let t = dias_desde_1970(2026, 11, 23) * 86_400 + 2 * 3600 + 30 * 60;
        assert_eq!(formatar_com(t, -180), ("22/11/2026".into(), "23:30".into()));
        assert_eq!(formatar_com(t, 330), ("23/11/2026".into(), "08:00".into()));
        // Ano bissexto e virada de século.
        for (a, m, d) in [(2024, 2, 29), (2000, 2, 29), (2100, 3, 1), (1969, 12, 31)] {
            assert_eq!(data_civil(dias_desde_1970(a, m, d)), (a, m, d));
        }
    }
}
