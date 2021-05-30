/*
 * Created on Mon Aug 03 2020
 *
 * This file is a part of Skytable
 * Skytable (formerly known as TerrabaseDB or Skybase) is a free and open-source
 * NoSQL database written by Sayan Nandan ("the Author") with the
 * vision to provide flexibility in data modelling without compromising
 * on performance, queryability or scalability.
 *
 * Copyright (c) 2020, Sayan Nandan <ohsayan@outlook.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 *
*/

//! # The Query Engine

use crate::coredb::CoreDB;
use crate::dbnet::connection::prelude::*;
use crate::gen_match;
use crate::protocol::responses;
use crate::protocol::Element;
use crate::{actions, admin};

mod tags {
    //! This module is a collection of tags/strings used for evaluating queries
    //! and responses
    /// `GET` action tag
    pub const TAG_GET: &'static str = "GET";
    /// `SET` action tag
    pub const TAG_SET: &'static str = "SET";
    /// `UPDATE` action tag
    pub const TAG_UPDATE: &'static str = "UPDATE";
    /// `DEL` action tag
    pub const TAG_DEL: &'static str = "DEL";
    /// `HEYA` action tag
    pub const TAG_HEYA: &'static str = "HEYA";
    /// `EXISTS` action tag
    pub const TAG_EXISTS: &'static str = "EXISTS";
    /// `MSET` action tag
    pub const TAG_MSET: &'static str = "MSET";
    /// `MGET` action tag
    pub const TAG_MGET: &'static str = "MGET";
    /// `MUPDATE` action tag
    pub const TAG_MUPDATE: &'static str = "MUPDATE";
    /// `SSET` action tag
    pub const TAG_SSET: &'static str = "SSET";
    /// `SDEL` action tag
    pub const TAG_SDEL: &'static str = "SDEL";
    /// `SUPDATE` action tag
    pub const TAG_SUPDATE: &'static str = "SUPDATE";
    /// `DBSIZE` action tag
    pub const TAG_DBSIZE: &'static str = "DBSIZE";
    /// `FLUSHDB` action tag
    pub const TAG_FLUSHDB: &'static str = "FLUSHDB";
    /// `USET` action tag
    pub const TAG_USET: &'static str = "USET";
    /// `KEYLEN` action tag
    pub const TAG_KEYLEN: &'static str = "KEYLEN";
    /// `MKSNAP` action tag
    pub const TAG_MKSNAP: &'static str = "MKSNAP";
    /// `LSKEYS` action tag
    pub const TAG_LSKEYS: &str = "LSKEYS";
}

/// Execute a simple(*) query
pub async fn execute_simple<T, Strm>(db: &CoreDB, con: &mut T, buf: Element) -> std::io::Result<()>
where
    T: ProtocolConnectionExt<Strm>,
    Strm: AsyncReadExt + AsyncWriteExt + Unpin + Send + Sync,
{
    let first = match buf.get_first() {
        Some(element) => element.to_ascii_uppercase(),
        None => return con.write_response(&**responses::groups::PACKET_ERR).await,
    };
    gen_match!(
        first,
        db,
        con,
        buf,
        tags::TAG_DEL => actions::del::del,
        tags::TAG_GET => actions::get::get,
        tags::TAG_HEYA => actions::heya::heya,
        tags::TAG_EXISTS => actions::exists::exists,
        tags::TAG_SET => actions::set::set,
        tags::TAG_MGET => actions::mget::mget,
        tags::TAG_MSET => actions::mset::mset,
        tags::TAG_UPDATE => actions::update::update,
        tags::TAG_MUPDATE => actions::mupdate::mupdate,
        tags::TAG_SSET => actions::strong::sset,
        tags::TAG_SDEL => actions::strong::sdel,
        tags::TAG_SUPDATE => actions::strong::supdate,
        tags::TAG_DBSIZE => actions::dbsize::dbsize,
        tags::TAG_FLUSHDB => actions::flushdb::flushdb,
        tags::TAG_USET => actions::uset::uset,
        tags::TAG_KEYLEN => actions::keylen::keylen,
        tags::TAG_MKSNAP => admin::mksnap::mksnap,
        tags::TAG_LSKEYS => actions::lskeys::lskeys
    );
    Ok(())
}

#[macro_export]
/// A match generator macro built specifically for the `crate::queryengine::execute_simple` function
///
/// **NOTE:** This macro needs _paths_ for both sides of the $x => $y, to produce something sensible
macro_rules! gen_match {
    ($pre:ident, $db:ident, $con:ident, $buf:ident, $($x:pat => $y:expr),*) => {
        let flat_array = if let crate::protocol::Element::FlatArray(array) = $buf {
            array
        } else {
            return $con.write_response(&**responses::groups::WRONGTYPE_ERR).await;
        };
        match $pre.as_str() {
            // First repeat over all the $x => $y patterns, passing in the variables
            // and adding .await calls and adding the `?`
            $(
                $x => $y($db, $con, flat_array).await?,
            )*
            // Now add the final case where no action is matched
            _ => {
                return $con.write_response(&**responses::groups::UNKNOWN_ACTION)
                .await;
            },
        }
    };
}
