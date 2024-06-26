use std::borrow::Cow;

macro_rules! format_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $Format:ident {
            $Custom:ident($CustomType:ty),
            $($Variant:ident = $str:literal,)*
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug)]
        $vis enum $Format {
            $Custom($CustomType),
            $($Variant,)*
        }

        impl $Format {
            pub fn str(&self) -> &str {
                match self {
                    $($Format::$Variant => $str,)*
                    $Format::$Custom(custom) => custom.as_ref(),
                }
            }
        }

        impl From<Cow<'static, str>> for $Format {
            fn from(s: Cow<'static, str>) -> $Format {
                match s {
                    $(s if s == $str => $Format::$Variant,)*
                    s => $Format::$Custom(s),
                }
            }
        }

        impl From<&'static str> for $Format {
            fn from(s: &'static str) -> $Format {
                $Format::from(Cow::Borrowed(s))
            }
        }

        impl From<String> for $Format {
            fn from(s: String) -> $Format {
                $Format::from(Cow::Owned(s))
            }
        }
    };
}

format_enum!(
    /// A format supported by CloudConvert.
    ///
    /// Serializes to the string value used in the CloudConvert API.
    pub enum Format {
        Custom(Cow<'static, str>),
        SevenZ = "7z",
        Ace = "ace",
        Alz = "alz",
        Arc = "arc",
        Arj = "arj",
        Bz = "bz",
        Bz2 = "bz2",
        Cab = "cab",
        Cpio = "cpio",
        Deb = "deb",
        Dmg = "dmg",
        Gz = "gz",
        Img = "img",
        Iso = "iso",
        Jar = "jar",
        Lha = "lha",
        Lz = "lz",
        Lzma = "lzma",
        Lzo = "lzo",
        Rar = "rar",
        Rpm = "rpm",
        Rz = "rz",
        Tar = "tar",
        Tar7z = "tar.7z",
        TarBz = "tar.bz",
        TarBz2 = "tar.bz2",
        TarGz = "tar.gz",
        TarLzo = "tar.lzo",
        TarXz = "tar.xz",
        TarZ = "tar.z",
        Tbz = "tbz",
        Tbz2 = "tbz2",
        Tgz = "tgz",
        Tz = "tz",
        Tzo = "tzo",
        Xz = "xz",
        Z = "z",
        Zip = "zip",
        //3g2 = "3g2",
        //3gp = "3gp",
        //3gpp = "3gpp",
        Aac = "aac",
        Ac3 = "ac3",
        Aif = "aif",
        Aifc = "aifc",
        Aiff = "aiff",
        Amr = "amr",
        Au = "au",
        Avi = "avi",
        Caf = "caf",
        Cavs = "cavs",
        Dv = "dv",
        Dvr = "dvr",
        Flac = "flac",
        Flv = "flv",
        Gif = "gif",
        M2ts = "m2ts",
        M4a = "m4a",
        M4b = "m4b",
        M4v = "m4v",
        Mkv = "mkv",
        Mod = "mod",
        Mov = "mov",
        Mp3 = "mp3",
        Mp4 = "mp4",
        Mpeg = "mpeg",
        Mpg = "mpg",
        Mts = "mts",
        Mxf = "mxf",
        Oga = "oga",
        Ogg = "ogg",
        Rm = "rm",
        Rmvb = "rmvb",
        Swf = "swf",
        Ts = "ts",
        Vob = "vob",
        Voc = "voc",
        Wav = "wav",
        Weba = "weba",
        Webm = "webm",
        Wma = "wma",
        Wmv = "wmv",
        Wtv = "wtv",
        Ai = "ai",
        Cdr = "cdr",
        Cgm = "cgm",
        Dwg = "dwg",
        Dxf = "dxf",
        Emf = "emf",
        Eps = "eps",
        Pdf = "pdf",
        Ps = "ps",
        Sk = "sk",
        Sk1 = "sk1",
        Svg = "svg",
        Svgz = "svgz",
        Vsd = "vsd",
        Wmf = "wmf",
        //3fr = "3fr",
        Abw = "abw",
        Arw = "arw",
        Avif = "avif",
        Azw = "azw",
        Azw3 = "azw3",
        Azw4 = "azw4",
        Bmp = "bmp",
        Cbc = "cbc",
        Cbr = "cbr",
        Cbz = "cbz",
        Chm = "chm",
        Cr2 = "cr2",
        Cr3 = "cr3",
        Crw = "crw",
        Csv = "csv",
        Dcr = "dcr",
        Djvu = "djvu",
        Dng = "dng",
        Doc = "doc",
        Docm = "docm",
        Docx = "docx",
        Dot = "dot",
        Dotx = "dotx",
        Dps = "dps",
        Epub = "epub",
        Erf = "erf",
        Et = "et",
        Fb2 = "fb2",
        Heic = "heic",
        Heif = "heif",
        Htm = "htm",
        Html = "html",
        Htmlz = "htmlz",
        Hwp = "hwp",
        Ico = "ico",
        Jfif = "jfif",
        Jpeg = "jpeg",
        Jpg = "jpg",
        Key = "key",
        Lit = "lit",
        Lrf = "lrf",
        Lwp = "lwp",
        Md = "md",
        Mobi = "mobi",
        Mos = "mos",
        Mrw = "mrw",
        Nef = "nef",
        Numbers = "numbers",
        Odd = "odd",
        Odg = "odg",
        Odp = "odp",
        Ods = "ods",
        Odt = "odt",
        Orf = "orf",
        Pages = "pages",
        Pdb = "pdb",
        Pef = "pef",
        Pml = "pml",
        Png = "png",
        Pot = "pot",
        Potx = "potx",
        Ppm = "ppm",
        Pps = "pps",
        Ppsx = "ppsx",
        Ppt = "ppt",
        Pptm = "pptm",
        Pptx = "pptx",
        Prc = "prc",
        Psd = "psd",
        Raf = "raf",
        Raw = "raw",
        Rb = "rb",
        Rst = "rst",
        Rtf = "rtf",
        Rw2 = "rw2",
        Snb = "snb",
        Tcr = "tcr",
        Tex = "tex",
        Tif = "tif",
        Tiff = "tiff",
        Txt = "txt",
        Txtz = "txtz",
        Webp = "webp",
        Wpd = "wpd",
        Wps = "wps",
        X3f = "x3f",
        Xcf = "xcf",
        Xls = "xls",
        Xlsm = "xlsm",
        Xlsx = "xlsx",
        Xps = "xps",
        Zabw = "zabw",
        Eot = "eot",
        Otf = "otf",
        Ttf = "ttf",
        Woff = "woff",
        Woff2 = "woff2",
        Icns = "icns",
    }
);

// Serialize to the string value
impl serde::Serialize for Format {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.str())
    }
}
