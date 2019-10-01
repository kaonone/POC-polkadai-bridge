import * as React from 'react';
import { Grid, Typography } from '@material-ui/core';

import { useSubscribable } from '~util/hooks';
import { useApi } from '~components/context';
import { Address } from '~components/Address';
import EthereumValidators from '~components/EthereumValidators';

import SendingForm from './SendingForm';

function EthereumToSubstrate() {
  const api = useApi();
  const [account, { error: accountError }] = useSubscribable(() => api.getEthAccount$(), []);

  return (
    <Grid container spacing={2}>
      <Grid item xs={12}>
        {!!accountError && <Typography color="error">{accountError}</Typography>}
        {account && <Address address={account} type="ethereum" />}
      </Grid>
      <Grid item xs={12}>
        <SendingForm />
      </Grid>
      <Grid item xs={12}>
        <EthereumValidators />
      </Grid>
    </Grid>
  );
}

export default EthereumToSubstrate;
