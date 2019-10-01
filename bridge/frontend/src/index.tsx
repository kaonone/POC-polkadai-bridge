import "reflect-metadata";
import Web3 from "web3";
import { ApiRx, WsProvider } from '@polkadot/api';
import * as React from "react";
import { render } from "react-dom";
import { MuiThemeProvider, createMuiTheme, CssBaseline } from '@material-ui/core';

import { Api } from "~/api";
import { SUBSTRATE_NODE_URL, SUBSTRATE_NODE_CUSTOM_TYPES } from '~env';

import App from '~components/App';
import { ApiContext } from "~/components/context";
import { ErrorBoundary } from "~/components/ErrorBoundary";

const theme = createMuiTheme({
  palette: {
    primary: {
      main: '#6931b6',
    },
  },
  overrides: {
    MuiFormHelperText: {
      root: {
        '&:empty': {
          display: 'none',
        },
      },
    },
  },
});

function Root() {
  // Detect if Web3 is found, if not, ask the user to install Metamask
  if (window.web3) {
    const web3 = new Web3(window.web3.currentProvider);
    const substrateApi = ApiRx.create({
      provider: new WsProvider(SUBSTRATE_NODE_URL),
      types: SUBSTRATE_NODE_CUSTOM_TYPES,
    });
    const api = new Api(web3, substrateApi);

    return (
      <ErrorBoundary>
        <MuiThemeProvider theme={theme}>
          <ApiContext.Provider value={api}>
            <CssBaseline />
            <App />
          </ApiContext.Provider>
        </MuiThemeProvider>
      </ErrorBoundary>
    );
  } else {
    return <div>You need to install Metamask</div>;
  }
}

const rootElement = document.getElementById("root");
render(<Root />, rootElement);
