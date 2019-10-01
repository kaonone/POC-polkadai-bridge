import React from "react";
import { Typography } from '@material-ui/core';

import { Balance } from '~components/Balance';

interface IProps {
  type: 'ethereum' | 'substrate';
  address: string;
  name?: string;
}

export function Address({ address, type, name }: IProps) {
  return (
    <>
      {!!name && <Typography variant="h5">{name}</Typography>}
      <Typography>Address: {address}</Typography>
      <Typography component="div">
        Balance: <Balance address={address} type={type} />
      </Typography>
    </>
  );
}
