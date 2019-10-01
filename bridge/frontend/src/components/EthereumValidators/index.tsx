import * as React from 'react';
import { Typography, LinearProgress, Button, Grid } from '@material-ui/core';
import { ETH_NETWORK_CONFIG } from '~env';
import { useSubscribable } from '~util/hooks';
import { useApi } from '~components/context';

function EthereumValidators() {
  const api = useApi();

  const [validators, { error, loaded }] = useSubscribable(() => api.getEthValidators$(), [], []);

  return (
    <div>
      <Typography variant='h4'>Ethereum Validators</Typography>
      {!loaded && !error && <LinearProgress />}
      {!!error && <Typography color="error">{error}</Typography>}
      {loaded && !error && (
        !validators.length
          ? <Typography color="error">Validators not found</Typography>
          : (
            <Grid container spacing={1}>
              {validators.map((validator, index) => (
                <Grid item key={validator}>
                  <Button
                    target="_blank"
                    rel="noopener noreferrer"
                    variant='outlined'
                    href={`${ETH_NETWORK_CONFIG.etherskanDomain}address/${validator}`}
                  >
                    # {index}
                  </Button>
                </Grid>
              ))}
            </Grid>
          )
      )}
    </div>
  );
}

export default EthereumValidators;
