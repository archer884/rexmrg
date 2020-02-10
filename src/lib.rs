use std::io;
use std::io::prelude::*;
use std::io::{BufReader};
use std::fs::File;
// use std::fs;
// use std::convert::TryInto;
use std::io::SeekFrom;
use std::f64::consts::PI;
// use std::ops::Range;
// use std::num::ParseIntError;
// use std::iter;

//https://www.nws.noaa.gov/oh/hrl/dmip/2/xmrgformat.html
// https://www.nws.noaa.gov/oh/hrl/misc/xmrg.pdf
// https://www.nws.noaa.gov/oh/hrl/dmip/2/src/read_xmrg2.c
// https://www.nws.noaa.gov/oh/hrl/distmodel/hrap.htm
// https://www.nws.noaa.gov/oh/hrl/gis/hrap/xmrgtolist.c
// https://www.nws.noaa.gov/oh/hrl/gis/hrap/xmrgtoasc.c
// HRAP https://www.nws.noaa.gov/oh/hrl/distmodel/hrap.htm
// HRAP function https://www.nws.noaa.gov/oh/hrl/dmip/lat_lon.txt

const XOR: usize = 0;
const YOR: usize = 1;
const COLUMNS: usize = 2;
const ROWS: usize = 3;

pub fn get_reader(path: &str) -> io::Result<BufReader<File>> {
    let file = File::open(path)?;
    Ok(BufReader::new(file))
}


pub fn read_b_int32<R: Read>(reader: &mut R) -> io::Result<i32> {

    let mut buffer = [0; 4];
    reader.read_exact(&mut buffer)?; // need error handling in case not 4 bytes?

    Ok(i32::from_be_bytes(buffer))
}


#[derive(Debug, Copy, Clone)]
pub enum Endian {
    Little,
    Big
}

// figure out if there is a way to consolidate some of these with generics. A type that all from_xx_bytes implement
impl Endian {
    pub fn read_int32<R: Read>(&self, reader: &mut R) -> io::Result<i32> {
        let mut buffer = [0; 4];
        reader.read_exact(&mut buffer)?; // need error handling in case not 4 bytes?

        match self {
            Endian::Big => Ok(i32::from_be_bytes(buffer)),
            Endian::Little => Ok(i32::from_le_bytes(buffer))
        }
    }

    pub fn read_int16<R: Read>(&self, reader: &mut R) -> io::Result<i16> {
        let mut buffer = [0; 2];
        reader.read_exact(&mut buffer)?; // need error handling in case not 4 bytes?

        match self {
            Endian::Big => Ok(i16::from_be_bytes(buffer)),
            Endian::Little => Ok(i16::from_le_bytes(buffer))
        }
    }

    pub fn read_u8<R: Read>(&self, reader: &mut R) -> io::Result<u8> {
        let mut buffer = [0; 1];
        reader.read_exact(&mut buffer)?; // need error handling in case not 4 bytes?

        match self {
            Endian::Big => Ok(u8::from_be_bytes(buffer)),
            Endian::Little => Ok(u8::from_le_bytes(buffer))
        }
    }
}


pub fn get_endian<R: Read>(reader: &mut R) -> io::Result<Endian> {
    let word = read_b_int32(reader);

    word.and_then(|int| {
        match int {
            16 => Ok(Endian::Big),
            _ => Ok(Endian::Little)
        }
    })
}



#[derive(Debug, Copy, Clone)]
pub struct ReadBytes {
    count: i32,
    endian: Endian
}

// figure out if there is a way to consolidate some of these into one using generics
// does self need to be a reference or can we consume it?
impl ReadBytes {

    pub fn new(count: i32, endian: Endian) -> Self {
        Self { count, endian }
    }

    pub fn iter_int32s<'a, R: Read>(self, reader: &'a mut R) -> impl Iterator<Item=io::Result<i32>> + 'a {
        (0..self.count).map(move |_| {
            self.endian.read_int32(reader)
        })
    }

    pub fn iter_int16s<'a, R: Read>(self, reader: &'a mut R) -> impl Iterator<Item=io::Result<i16>> + 'a {
        (0..self.count).map(move |_| {
            self.endian.read_int16(reader)
        })
    }

    pub fn iter_u8s<'a, R: Read>(self, reader: &'a mut R) -> impl Iterator<Item=io::Result<u8>> + 'a {
        (0..self.count).map(move |_| {
            self.endian.read_u8(reader)
        })
    }

    // The following are convenience methods so you don't need to write collect::<io::Result<Vec<TYPE>>>() when you just want the bytes in a Vec

    pub fn read_int32s<R: Read>(self, reader: &mut R) -> io::Result<Vec<i32>> {
        self.iter_int32s(reader).collect()
    }

    pub fn read_int16s<R: Read>(self, reader: &mut R) -> io::Result<Vec<i16>> {
        self.iter_int16s(reader).collect()
    }

    pub fn read_u8s<R: Read>(self, reader: &mut R) -> io::Result<Vec<u8>> {
        self.iter_u8s(reader).collect()
    }
}

// see the second record section of https://www.nws.noaa.gov/oh/hrl/misc/xmrg.pdf
#[derive(Debug, Eq, PartialEq)]
pub enum XmrgVersion {
    Pre1997,
    Build4_2,
    Build5_2_2,
}


pub fn get_xmrg_version(byte_count: i32, max_x: i32) -> Option<XmrgVersion> {
    match byte_count {
        66 => Some(XmrgVersion::Build5_2_2),
        38 => Some(XmrgVersion::Build4_2), // a 37 byte version may be valid. Consider adding
        n if n == max_x * 2 => Some(XmrgVersion::Pre1997),
        _ => None
    }
}


// if a data point is negative, represent as -999 (no data), if positive, divide by 100 to represent in millimeters
// data points are represented as a 100th of a milimeter. .001mm is represented as 1 in a xmrg data point, dividing by 100 gets us to .001
pub fn to_mm(data_point: i16) -> f64 {
    if data_point < 0 {
        -999.0
    } else {
        data_point as f64 / 100.0
    }
}

pub fn process_row<R: Read + Seek>(read_bytes: ReadBytes, reader: &mut R) -> io::Result<Vec<f64>> {
    reader.seek(SeekFrom::Current(4))?;

    let result = 
        read_bytes
            .iter_int16s(reader)
            .map(|res| res.map(to_mm))
            .collect();

    reader.seek(SeekFrom::Current(4))?;

    result
}

pub fn read_xmrg(path: &str) -> io::Result<Vec<Vec<f64>>> {
    let mut reader = get_reader(path)?;
    let endian = get_endian(&mut reader)?;

    let header = ReadBytes::new(4,endian).read_int32s(&mut reader)?;
    reader.seek(SeekFrom::Current(4))?;

    let record_2_bytes = endian.read_int32(&mut reader)?;

    let xmrg_version = get_xmrg_version(record_2_bytes, header[COLUMNS]);

    let row_reader = ReadBytes::new(header[COLUMNS], endian);

    xmrg_version.map_or(Ok(Vec::new()), |version| {
        match version {
            XmrgVersion::Pre1997 => {
                reader.seek(SeekFrom::Start(24))?; // set reader to position just after header (4 bytes + 16 byte header + 4 bytes = 24) 

                (0..header[ROWS]).map(|_| {
                    process_row(row_reader, &mut reader)
                })
                .collect()
            },
            _ => Ok(Vec::new()) // not implemented
        }
    })
}


#[derive(Debug, Copy, Clone)]
pub struct DateSegments {
    month: i32,
    day: i32,
    year: i32,
    hour: i32 // in 24 hour time
}

impl DateSegments {
    // look into better strategy than indexing
    pub fn from_chars(chars: &str) -> Self {
        // assert_eq!(chars.len(), 10);
        DateSegments {
            month: chars[0..2].parse::<i32>().unwrap_or_default(),
            day: chars[2..4].parse::<i32>().unwrap_or_default(),
            year: chars[4..8].parse::<i32>().unwrap_or_default(),
            hour: chars[8..10].parse::<i32>().unwrap_or_default(),
        }
    }
}

pub fn read_old_xmrg_date(path: &str) -> DateSegments {
    let date_chars = path.chars()
        .skip_while(|c| *c < '0' || *c > '9')
        .collect::<String>();

    DateSegments::from_chars(&date_chars)
}

#[derive(Debug, Copy, Clone)]
pub struct Point {
    x: f64,
    y: f64
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

pub struct CoordinateGenerator {
    start_x: i32,
    current_x: i32,
    current_y: i32,
    x_end: i32,
    y_end: i32
}

impl Iterator for CoordinateGenerator {
    type Item = Point;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.current_x += 1;
        if self.current_x == self.x_end {
            self.current_x = self.start_x;
            self.current_y += 1;
        }

        if self.current_y != self.y_end {
            Some(hrap_to_latlon(f64::from(self.current_x), f64::from(self.current_y)))
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Header {
    xor: i32,
    yor: i32,
    columns: i32,
    rows: i32
}

impl Header {
    pub fn from_vec(vec: Vec<i32>) -> Self {
        Self {
            xor: vec[XOR],
            yor: vec[YOR],
            columns: vec[COLUMNS],
            rows: vec[ROWS]
        }
    }

    pub fn generate_coordinates(&self) -> Vec<Vec<Point>> {
        (self.yor..self.rows).map(|y| {
            (self.xor..self.columns).map(|x| {
                hrap_to_latlon(f64::from(x), f64::from(y))
            })
            .collect()
        })
        .collect()
    }
}

impl IntoIterator for Header {
    type Item = Point;
    type IntoIter = CoordinateGenerator;

    fn into_iter(self) -> Self::IntoIter {
        CoordinateGenerator {
            start_x: self.xor,
            current_x: self.xor - 1,
            current_y: self.yor,
            x_end: self.xor + self.columns,
            y_end: self.yor + self.rows
        }
    }
}

// HRAP : https://www.nws.noaa.gov/oh/hrl/nwsrfs/users_manual/part2/_pdf/21hrapgrid.pdf
// positive longitude values are West, Positive latitude North
// derived from https://www.nws.noaa.gov/oh/hrl/dmip/lat_lon.txt
pub fn hrap_to_latlon(x: f64, y: f64) -> Point {
    let earthr = 6371.2;
    let stlon = 105.0;
    let raddeg = 180.0 / PI;
    let xmesh = 4.7625;
    let tlat = 60.0 / raddeg;

    let _x = x - 401.0; // >
    let _y = y - 1601.0; // >

    let rr = (_x * _x) + (_y * _y);

    let gi = (earthr * (1.0 + tlat.sin())) / xmesh;
    let _gi = gi * gi;

    let rlat = ((_gi - rr) / (_gi + rr)).asin() * raddeg;

    let mut ang = _y.atan2(_x) * raddeg;

    // let if (ang.lt.0.) ang = ang + 360.0;
    ang += if ang < 0.0 { 360.0 } else { 0.0 }; 

    let mut rlon = 270.0 + stlon - ang;

    // let if(rlon.lt.0.) rlon=rlon+360.0;
    rlon += if rlon < 0.0 { 360.0 } else { 0.0 };

    // let if(rlon.gt.360.0) rlon = rlon - 360.0;
    rlon -= if rlon > 360.0 { 360.0 } else { 0.0 };

    Point::new(rlon, rlat)
}

pub struct Feature {
    point: Point,
    value: f64
}

impl Feature {

    pub fn new(point: Point, value: f64) -> Self {
        Feature { point, value }
    }

    pub fn csv_row(&self) -> String {
        // long lat value
        String::from(format!("{},{},{}", self.point.x, self.point.y, self.value))
    }
}

pub struct XmrgData {
    header: Header,
    values: Vec<Vec<f64>>,
}

impl XmrgData {

    pub fn new(header: Header, values: Vec<Vec<f64>>) -> Self {
        XmrgData { header, values }
    }

    // https://github.com/rust-lang/rfcs/blob/master/text/1951-expand-impl-trait.md#scoping-for-type-and-lifetime-parameters
    // pub fn generate_features<'a>(&'a self) -> impl Iterator<Item=Feature> + 'a {
    pub fn generate_features(&self) -> impl Iterator<Item=Feature> + '_ {

        self.values.iter()
            .flat_map(|vec| vec.iter())
            .zip(self.header.into_iter())
            .map(|(value, point)| {
                Feature::new(point, *value)
            })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_be_ne_test() {
        let buff = [0b10101010, 0b11100101];

        assert_eq!(u16::from_be_bytes(buff), 0b1010101011100101);
        assert_eq!(u16::from_le_bytes(buff), 0b1110010110101010);
        assert_eq!(u16::from_ne_bytes(buff), 0b1110010110101010);
    }

    #[test]
    fn get_xmrg_version_test() {
        let columns = 100;
        let first_byte_count = 66;
        let second_byte_count = 38;

        let v1 = get_xmrg_version(first_byte_count, columns);
        assert_eq!(v1, Some(XmrgVersion::Build5_2_2));

        let v2 = get_xmrg_version(second_byte_count, columns);
        assert_eq!(v2, Some(XmrgVersion::Build4_2));

        let v3 = get_xmrg_version(columns * 2, columns);
        assert_eq!(v3, Some(XmrgVersion::Pre1997));
    }

    #[test]
    fn hrap_to_latlon_test() { // write a better test here
        let hrap_x = 367.0;
        let hrap_y = 263.0;

        let point = hrap_to_latlon(hrap_x, hrap_y);

        assert!(point.x > 106.0 && point.x < 107.0);
        assert!(point.y > 33.0 && point.y < 34.0);
    }

    #[test]
    fn read_old_xmrg_date_test() {
        let path = "xmrg0506199516z.gz";

        let data_segments = read_old_xmrg_date(&path);

        assert_eq!(data_segments.month, 5);
        assert_eq!(data_segments.day, 6);       
        assert_eq!(data_segments.year, 1995);
        assert_eq!(data_segments.hour, 16);
    }
}


// pub fn read_xmrg(file_path: &str) -> io::Result<()> {
//     let file = File::open(file_path)?;
//     let mut reader = BufReader::new(file);
//     let mut num_bytes = [0; 1];
    
//     let mut handle = reader.take(1);
    
//     handle.read(&mut num_bytes)?;
//     let _needs_reversal = need_byte_reversal(num_bytes[0].try_into().unwrap());
    
//     Ok(())
// }

// impl ByteReader for Vec<u8> {
//     fn read_int32(&self) -> i32 {
//         i32::from_le_bytes(*self)
//     }
// }