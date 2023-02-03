//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
use crate::{RCodec, WCodec, Zenoh080, Zenoh080Header};
use core::convert::TryFrom;
use zenoh_buffers::{
    reader::{DidntRead, Reader},
    writer::{DidntWrite, Writer},
};
use zenoh_protocol::{
    common::{imsg, ZExtUnknown},
    core::{whatami::WhatAmIMatcher, ZenohId},
    scouting::{
        id,
        scout::{flag, Scout},
    },
};

impl<W> WCodec<&Scout, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = Result<(), DidntWrite>;

    fn write(self, writer: &mut W, x: &Scout) -> Self::Output {
        // Header
        let mut header = id::SCOUT;
        if x.zid.is_some() {
            header |= flag::I;
        }
        self.write(&mut *writer, header)?;

        // Body
        self.write(&mut *writer, x.version)?;
        let what: u8 = x.what.into();
        self.write(&mut *writer, what & 0b111)?;
        if let Some(zid) = x.zid.as_ref() {
            self.write(&mut *writer, zid)?;
        }

        Ok(())
    }
}

impl<R> RCodec<Scout, &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<Scout, Self::Error> {
        let header: u8 = self.read(&mut *reader)?;
        let codec = Zenoh080Header::new(header);

        codec.read(reader)
    }
}

impl<R> RCodec<Scout, &mut R> for Zenoh080Header
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<Scout, Self::Error> {
        if imsg::mid(self.header) != id::SCOUT {
            return Err(DidntRead);
        }

        // Body
        let version: u8 = self.codec.read(&mut *reader)?;
        let flags: u8 = self.codec.read(&mut *reader)?;
        let what = WhatAmIMatcher::try_from(flags & 0b111).map_err(|_| DidntRead)?;
        let zid = if imsg::has_flag(self.header, flag::I) {
            let zid: ZenohId = self.codec.read(&mut *reader)?;
            Some(zid)
        } else {
            None
        };

        // Extensions
        let mut has_extensions = imsg::has_flag(self.header, flag::Z);
        while has_extensions {
            let (_, more): (ZExtUnknown, bool) = self.codec.read(&mut *reader)?;
            has_extensions = more;
        }

        Ok(Scout { version, what, zid })
    }
}
