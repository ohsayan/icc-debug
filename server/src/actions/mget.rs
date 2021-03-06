/*
 * Created on Thu Aug 27 2020
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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 *
*/

use crate::dbnet::connection::prelude::*;
use crate::queryengine::ActionIter;
use crate::resp::BytesWrapper;
use bytes::Bytes;
use skytable::RespCode;

action!(
    /// Run an `MGET` query
    ///
    fn mget(handle: &crate::corestore::Corestore, con: &mut T, act: ActionIter) {
        crate::err_if_len_is!(act, con, eq 0);
        con.write_array_length(act.len()).await?;
        for key in act {
            let res: Option<Bytes> = match kve!(con, handle).get(key) {
                Ok(v) => v.map(|b| b.get_blob().clone()),
                Err(_) => None,
            };
            if let Some(value) = res {
                // Good, we got the value, write it off to the stream
                con.write_response(BytesWrapper(value)).await?;
            } else {
                // Ah, couldn't find that key
                con.write_response(RespCode::NotFound).await?;
            }
        }
        Ok(())
    }
);
